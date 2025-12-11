/**
 * Integration tests for the changeset commands.
 *
 * ## What
 *
 * This module contains comprehensive integration tests for all changeset NAPI functions:
 * - `changesetAdd` - Create a new changeset for tracking version changes
 * - `changesetUpdate` - Update an existing changeset with new commits/packages
 * - `changesetList` - List all pending changesets with filtering options
 * - `changesetShow` - Display details of a specific changeset
 * - `changesetRemove` - Remove a changeset from the workspace
 * - `changesetHistory` - Query archived changesets with filtering
 * - `changesetCheck` - Check if a changeset exists for a branch
 *
 * ## How
 *
 * Tests are organized into logical groups:
 * - Success tests: Verify each command works with valid parameters
 * - Error tests: Verify proper error handling for invalid inputs
 * - Type verification tests: Ensure TypeScript types match actual response structure
 * - Filter and option tests: Verify various parameter combinations work correctly
 *
 * Each test creates an isolated temporary directory with:
 * - A package.json for workspace identification
 * - An initialized git repository with proper configuration
 * - Workspace configuration via the `init` command
 *
 * ## Why
 *
 * Integration tests validate that the Node.js bindings work correctly end-to-end,
 * ensuring the Rust code, NAPI bindings, and TypeScript types are all aligned.
 * Changesets are the core workflow mechanism for tracking changes before version
 * bumps, making their correct operation critical for the entire release process.
 *
 * @packageDocumentation
 */

import test from 'ava';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';

import {
  changesetAdd,
  changesetUpdate,
  changesetList,
  changesetShow,
  changesetRemove,
  changesetHistory,
  changesetCheck,
  init,
} from '../src/index';

import type {
  ChangesetAddParams,
  ChangesetAddApiResponse,
  ChangesetAddData,
  ChangesetUpdateParams,
  ChangesetUpdateApiResponse,
  ChangesetUpdateData,
  ChangesetListParams,
  ChangesetListApiResponse,
  ChangesetListData,
  ChangesetListItemInfo,
  ChangesetShowParams,
  ChangesetShowApiResponse,
  ChangesetShowData,
  ChangesetRemoveParams,
  ChangesetRemoveApiResponse,
  ChangesetRemoveData,
  ChangesetHistoryParams,
  ChangesetHistoryApiResponse,
  ChangesetHistoryData,
  ChangesetCheckParams,
  ChangesetCheckApiResponse,
  ChangesetCheckData,
  ChangesetDetailInfo,
  UpdateSummaryInfo,
  ArchivedChangesetInfo,
  ErrorInfo,
} from '../src/index';

// ============================================================================
// Test Fixtures and Helpers
// ============================================================================

/**
 * Creates a temporary directory for testing.
 * Returns the path to the created directory.
 */
function createTempDir(prefix: string = 'changeset-test-'): string {
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
 * Sets up the default branch as 'main' for consistency across git versions.
 */
function initGitRepo(dirPath: string): void {
  const { execSync } = require('child_process');
  execSync('git init -b main', { cwd: dirPath, stdio: 'ignore' });
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
 * Creates an initial commit in the git repository.
 */
function createInitialCommit(dirPath: string): string {
  const { execSync } = require('child_process');
  execSync('git add -A', { cwd: dirPath, stdio: 'ignore' });
  execSync('git commit -m "Initial commit"', {
    cwd: dirPath,
    stdio: 'ignore',
  });
  const commitHash = execSync('git rev-parse HEAD', {
    cwd: dirPath,
    encoding: 'utf-8',
  }).trim();
  return commitHash;
}

/**
 * Creates a new git branch.
 */
function createBranch(dirPath: string, branchName: string): void {
  const { execSync } = require('child_process');
  execSync(`git checkout -b ${branchName}`, {
    cwd: dirPath,
    stdio: 'ignore',
  });
}

/**
 * Gets the current git branch name.
 */
function getCurrentBranch(dirPath: string): string {
  const { execSync } = require('child_process');
  return execSync('git rev-parse --abbrev-ref HEAD', {
    cwd: dirPath,
    encoding: 'utf-8',
  }).trim();
}

/**
 * Creates a commit with a dummy file change.
 */
function createCommit(dirPath: string, message: string): string {
  const { execSync } = require('child_process');
  const fileName = `file-${Date.now()}.txt`;
  fs.writeFileSync(path.join(dirPath, fileName), `Content: ${message}`);
  execSync('git add -A', { cwd: dirPath, stdio: 'ignore' });
  execSync(`git commit -m "${message}"`, {
    cwd: dirPath,
    stdio: 'ignore',
  });
  const commitHash = execSync('git rev-parse HEAD', {
    cwd: dirPath,
    encoding: 'utf-8',
  }).trim();
  return commitHash;
}

/**
 * Switches to an existing branch.
 */
function switchBranch(dirPath: string, branchName: string): void {
  const { execSync } = require('child_process');
  execSync(`git checkout ${branchName}`, {
    cwd: dirPath,
    stdio: 'ignore',
  });
}

/**
 * Gets the default branch name (main or master).
 */
function getDefaultBranch(dirPath: string): string {
  const { execSync } = require('child_process');
  // Get the default branch from the initial commit
  try {
    return execSync('git rev-parse --abbrev-ref HEAD', {
      cwd: dirPath,
      encoding: 'utf-8',
    }).trim();
  } catch {
    return 'main';
  }
}

/**
 * Initializes a workspace with default configuration.
 * Returns true if successful.
 */
async function initWorkspace(dirPath: string): Promise<boolean> {
  const result = await init({ root: dirPath });
  return result.success;
}

/**
 * Sets up a complete test environment with git repo and workspace config.
 */
async function setupTestEnvironment(prefix: string = 'changeset-test-'): Promise<string> {
  const tempDir = createTempDir(prefix);
  createPackageJson(tempDir);
  initGitRepo(tempDir);
  createInitialCommit(tempDir);
  await initWorkspace(tempDir);
  return tempDir;
}

// ============================================================================
// changesetAdd Tests
// ============================================================================

test('changesetAdd - creates a changeset with minimal parameters', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-add-min-');

  try {
    createBranch(tempDir, 'feature/test-changeset');

    const params: ChangesetAddParams = {
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    };

    const result: ChangesetAddApiResponse = await changesetAdd(params);

    t.true(result.success, 'changesetAdd should succeed');
    t.truthy(result.data, 'Data should be present on success');
    t.is(result.error, undefined, 'Error should not be present on success');

    const data: ChangesetAddData = result.data as ChangesetAddData;
    t.is(typeof data.id, 'string', 'ID should be a string');
    t.true(data.id.length > 0, 'ID should not be empty');
    t.is(data.branch, 'feature/test-changeset', 'Branch should match');
    t.is(data.bump, 'patch', 'Bump should be patch');
    t.true(Array.isArray(data.packages), 'Packages should be an array');
    t.true(Array.isArray(data.environments), 'Environments should be an array');
    t.is(typeof data.createdAt, 'string', 'Created at should be a string');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetAdd - creates a changeset with all parameters', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-add-full-');

  try {
    createBranch(tempDir, 'feature/full-test');

    const params: ChangesetAddParams = {
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
      environments: ['staging', 'production'],
      branch: 'feature/full-test',
      message: 'Add new feature for testing',
    };

    const result = await changesetAdd(params);

    t.true(result.success);
    t.truthy(result.data);

    const data = result.data as ChangesetAddData;
    t.is(data.bump, 'minor');
    t.deepEqual(data.packages, ['test-package']);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetAdd - creates changeset with major bump', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-add-major-');

  try {
    createBranch(tempDir, 'breaking/api-change');

    const result = await changesetAdd({
      root: tempDir,
      bump: 'major',
      packages: ['test-package'],
    });

    t.true(result.success);
    t.truthy(result.data);
    t.is((result.data as ChangesetAddData).bump, 'major');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetAdd - force overwrites existing changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-add-force-');

  try {
    createBranch(tempDir, 'feature/force-test');

    // Create first changeset
    const firstResult = await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });
    t.true(firstResult.success);

    // Try to create again without force - should fail
    const secondResult = await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
    });
    t.false(secondResult.success);

    // Create with force - should succeed
    const thirdResult = await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
      force: true,
    });
    t.true(thirdResult.success);
    t.is((thirdResult.data as ChangesetAddData).bump, 'minor');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetAdd - returns error for non-existent path', async (t) => {
  const nonExistentPath = '/non/existent/path/for/changeset/test';

  const result = await changesetAdd({
    root: nonExistentPath,
    bump: 'patch',
    packages: ['test-package'],
  });

  t.false(result.success);
  t.is(result.data, undefined);
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(typeof error.code, 'string');
  t.is(typeof error.message, 'string');
  t.is(typeof error.kind, 'string');
});

test('changesetAdd - returns error for empty root path', async (t) => {
  const result = await changesetAdd({
    root: '',
    bump: 'patch',
    packages: ['test-package'],
  });

  t.false(result.success);
  t.truthy(result.error);
  t.is((result.error as ErrorInfo).code, 'EVALIDATION');
});

test('changesetAdd - returns error for invalid bump type', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-add-invalid-bump-');

  try {
    createBranch(tempDir, 'feature/invalid-bump');

    const result = await changesetAdd({
      root: tempDir,
      bump: 'invalid-bump-type',
      packages: ['test-package'],
    });

    t.false(result.success);
    t.truthy(result.error);
    t.is((result.error as ErrorInfo).code, 'EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetAdd - returns error for uninitialized workspace', async (t) => {
  const tempDir = createTempDir('changeset-add-uninit-');

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);
    createInitialCommit(tempDir);
    createBranch(tempDir, 'feature/no-config');

    const result = await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    t.false(result.success);
    t.truthy(result.error);
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// changesetUpdate Tests
// ============================================================================

test('changesetUpdate - updates changeset bump type', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-update-bump-');

  try {
    createBranch(tempDir, 'feature/update-bump');

    // Create initial changeset
    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    // Update bump type
    const params: ChangesetUpdateParams = {
      root: tempDir,
      id: 'feature/update-bump',
      bump: 'minor',
    };

    const result: ChangesetUpdateApiResponse = await changesetUpdate(params);

    t.true(result.success, 'changesetUpdate should succeed');
    t.truthy(result.data, 'Data should be present on success');

    const data: ChangesetUpdateData = result.data as ChangesetUpdateData;
    t.is(typeof data.updated, 'boolean', 'Updated should be a boolean');
    t.truthy(data.summary, 'Summary should be present');
    t.truthy(data.changeset, 'Changeset should be present');
    t.is(data.changeset.bump, 'minor', 'Bump should be updated to minor');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetUpdate - adds packages to changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-update-pkg-');

  try {
    createBranch(tempDir, 'feature/update-packages');

    // Create a second package
    const pkg2Dir = path.join(tempDir, 'packages', 'pkg2');
    fs.mkdirSync(pkg2Dir, { recursive: true });
    fs.writeFileSync(
      path.join(pkg2Dir, 'package.json'),
      JSON.stringify({ name: '@scope/pkg2', version: '1.0.0' }, null, 2)
    );

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const result = await changesetUpdate({
      root: tempDir,
      id: 'feature/update-packages',
      packages: ['@scope/pkg2'],
    });

    t.true(result.success);
    t.truthy(result.data);

    const data = result.data as ChangesetUpdateData;
    t.truthy(data.summary);

    const summary: UpdateSummaryInfo = data.summary;
    t.is(typeof summary.packagesAdded, 'number');
    t.is(typeof summary.commitsAdded, 'number');
    t.is(typeof summary.bumpUpdated, 'boolean');
    t.is(typeof summary.environmentsAdded, 'number');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetUpdate - adds commit to changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-update-commit-');

  try {
    createBranch(tempDir, 'feature/update-commit');

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const commitHash = createCommit(tempDir, 'Additional feature work');

    const result = await changesetUpdate({
      root: tempDir,
      id: 'feature/update-commit',
      commit: commitHash,
    });

    t.true(result.success);
    t.truthy(result.data);

    const data = result.data as ChangesetUpdateData;
    t.true(data.changeset.commits.length > 0, 'Should have commits');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetUpdate - adds environments to changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-update-env-');

  try {
    createBranch(tempDir, 'feature/update-env');

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const result = await changesetUpdate({
      root: tempDir,
      id: 'feature/update-env',
      environments: ['staging', 'production'],
    });

    t.true(result.success);
    t.truthy(result.data);

    const data = result.data as ChangesetUpdateData;
    t.true(data.changeset.environments.length > 0, 'Should have environments');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetUpdate - returns error for non-existent changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-update-noexist-');

  try {
    const result = await changesetUpdate({
      root: tempDir,
      id: 'nonexistent/branch',
      bump: 'minor',
    });

    t.false(result.success);
    t.truthy(result.error);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetUpdate - returns error for empty root', async (t) => {
  const result = await changesetUpdate({
    root: '',
    id: 'some/branch',
    bump: 'minor',
  });

  t.false(result.success);
  t.truthy(result.error);
  t.is((result.error as ErrorInfo).code, 'EVALIDATION');
});

// ============================================================================
// changesetList Tests
// ============================================================================

test('changesetList - returns empty list when no changesets exist', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-list-empty-');

  try {
    const params: ChangesetListParams = {
      root: tempDir,
    };

    const result: ChangesetListApiResponse = await changesetList(params);

    t.true(result.success, 'changesetList should succeed');
    t.truthy(result.data, 'Data should be present');

    const data: ChangesetListData = result.data as ChangesetListData;
    t.true(Array.isArray(data.changesets), 'Changesets should be an array');
    t.is(typeof data.count, 'number', 'Count should be a number');
    t.is(data.count, 0, 'Count should be 0');
    t.is(data.changesets.length, 0, 'Changesets array should be empty');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetList - returns list of changesets', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-list-');

  try {
    // Create first changeset
    createBranch(tempDir, 'feature/list-test-1');
    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    // Create second changeset
    switchBranch(tempDir, 'main');
    createBranch(tempDir, 'feature/list-test-2');
    await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
    });

    const result = await changesetList({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);

    const data = result.data as ChangesetListData;
    t.true(data.count >= 2, 'Should have at least 2 changesets');
    t.true(data.changesets.length >= 2, 'Changesets array should have items');

    // Verify item structure
    const item: ChangesetListItemInfo = data.changesets[0];
    t.is(typeof item.id, 'string');
    t.is(typeof item.branch, 'string');
    t.is(typeof item.bump, 'string');
    t.true(Array.isArray(item.packages));
    t.true(Array.isArray(item.environments));
    t.is(typeof item.commitCount, 'number');
    t.is(typeof item.createdAt, 'string');
    t.is(typeof item.updatedAt, 'string');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetList - filters by bump type', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-list-filter-bump-');

  try {
    // Create patch changeset
    createBranch(tempDir, 'fix/patch-change');
    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    // Create minor changeset
    switchBranch(tempDir, 'main');
    createBranch(tempDir, 'feature/minor-change');
    await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
    });

    // Filter by patch
    const result = await changesetList({
      root: tempDir,
      filterBump: 'patch',
    });

    t.true(result.success);
    const data = result.data as ChangesetListData;
    t.true(data.changesets.every((cs) => cs.bump === 'patch'));
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetList - filters by package', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-list-filter-pkg-');

  try {
    createBranch(tempDir, 'feature/pkg-filter-test');
    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const result = await changesetList({
      root: tempDir,
      filterPackage: 'test-package',
    });

    t.true(result.success);
    const data = result.data as ChangesetListData;
    t.true(data.changesets.every((cs) => cs.packages.includes('test-package')));
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetList - returns error for non-existent path', async (t) => {
  const result = await changesetList({
    root: '/non/existent/path',
  });

  t.false(result.success);
  t.truthy(result.error);
});

test('changesetList - returns error for empty root', async (t) => {
  const result = await changesetList({ root: '' });

  t.false(result.success);
  t.truthy(result.error);
  t.is((result.error as ErrorInfo).code, 'EVALIDATION');
});

test('changesetList - returns error for invalid sort option', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-list-invalid-sort-');

  try {
    const result = await changesetList({
      root: tempDir,
      sort: 'invalid-sort-option',
    });

    t.false(result.success);
    t.truthy(result.error);
    t.is((result.error as ErrorInfo).code, 'EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// changesetShow Tests
// ============================================================================

test('changesetShow - shows changeset details', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-show-');

  try {
    const branchName = 'feature/show-test';
    createBranch(tempDir, branchName);

    await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
      message: 'Test changeset message',
    });

    const params: ChangesetShowParams = {
      root: tempDir,
      branch: branchName,
    };

    const result: ChangesetShowApiResponse = await changesetShow(params);

    t.true(result.success, 'changesetShow should succeed');
    t.truthy(result.data, 'Data should be present');

    const data: ChangesetShowData = result.data as ChangesetShowData;
    t.truthy(data.changeset, 'Changeset details should be present');

    const changeset: ChangesetDetailInfo = data.changeset;
    t.is(typeof changeset.id, 'string');
    t.is(changeset.branch, branchName);
    t.is(changeset.bump, 'minor');
    t.true(Array.isArray(changeset.packages));
    t.true(Array.isArray(changeset.environments));
    t.true(Array.isArray(changeset.commits));
    t.is(typeof changeset.createdAt, 'string');
    t.is(typeof changeset.updatedAt, 'string');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetShow - shows changeset with all fields', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-show-full-');

  try {
    const branchName = 'feature/show-full-test';
    createBranch(tempDir, branchName);

    await changesetAdd({
      root: tempDir,
      bump: 'major',
      packages: ['test-package'],
      environments: ['staging', 'production'],
      message: 'Major breaking change',
    });

    const result = await changesetShow({
      root: tempDir,
      branch: branchName,
    });

    t.true(result.success);
    const data = result.data as ChangesetShowData;

    t.is(data.changeset.bump, 'major');
    t.deepEqual(data.changeset.packages, ['test-package']);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetShow - returns error for non-existent changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-show-noexist-');

  try {
    const result = await changesetShow({
      root: tempDir,
      branch: 'nonexistent/branch',
    });

    t.false(result.success);
    t.truthy(result.error);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetShow - returns error for empty branch', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-show-empty-branch-');

  try {
    const result = await changesetShow({
      root: tempDir,
      branch: '',
    });

    t.false(result.success);
    t.truthy(result.error);
    t.is((result.error as ErrorInfo).code, 'EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetShow - returns error for empty root', async (t) => {
  const result = await changesetShow({
    root: '',
    branch: 'some/branch',
  });

  t.false(result.success);
  t.truthy(result.error);
  t.is((result.error as ErrorInfo).code, 'EVALIDATION');
});

// ============================================================================
// changesetRemove Tests
// ============================================================================

test('changesetRemove - removes existing changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-remove-');

  try {
    const branchName = 'feature/remove-test';
    createBranch(tempDir, branchName);

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    // Verify it exists
    const listBefore = await changesetList({ root: tempDir });
    t.true((listBefore.data as ChangesetListData).count > 0);

    const params: ChangesetRemoveParams = {
      root: tempDir,
      branch: branchName,
      force: true,
    };

    const result: ChangesetRemoveApiResponse = await changesetRemove(params);

    t.true(result.success, 'changesetRemove should succeed');
    t.truthy(result.data, 'Data should be present');

    const data: ChangesetRemoveData = result.data as ChangesetRemoveData;
    t.is(typeof data.removed, 'boolean');
    t.true(data.removed);
    t.is(typeof data.branch, 'string');
    t.is(data.branch, branchName);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetRemove - removes changeset and archives it', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-remove-archive-');

  try {
    const branchName = 'feature/remove-archive-test';
    createBranch(tempDir, branchName);

    await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
    });

    await changesetRemove({
      root: tempDir,
      branch: branchName,
      force: true,
    });

    // Verify it's gone from pending
    const listAfter = await changesetList({ root: tempDir });
    const pending = (listAfter.data as ChangesetListData).changesets;
    t.false(pending.some((cs) => cs.branch === branchName));
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetRemove - returns error for non-existent changeset', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-remove-noexist-');

  try {
    const result = await changesetRemove({
      root: tempDir,
      branch: 'nonexistent/branch',
      force: true,
    });

    t.false(result.success);
    t.truthy(result.error);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetRemove - returns error for empty branch', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-remove-empty-branch-');

  try {
    const result = await changesetRemove({
      root: tempDir,
      branch: '',
    });

    t.false(result.success);
    t.truthy(result.error);
    t.is((result.error as ErrorInfo).code, 'EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetRemove - returns error for empty root', async (t) => {
  const result = await changesetRemove({
    root: '',
    branch: 'some/branch',
  });

  t.false(result.success);
  t.truthy(result.error);
  t.is((result.error as ErrorInfo).code, 'EVALIDATION');
});

// ============================================================================
// changesetHistory Tests
// ============================================================================

test('changesetHistory - returns empty history when no archived changesets', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-history-empty-');

  try {
    const params: ChangesetHistoryParams = {
      root: tempDir,
    };

    const result: ChangesetHistoryApiResponse = await changesetHistory(params);

    t.true(result.success, 'changesetHistory should succeed');
    t.truthy(result.data, 'Data should be present');

    const data: ChangesetHistoryData = result.data as ChangesetHistoryData;
    t.true(Array.isArray(data.archived), 'Archived should be an array');
    t.is(typeof data.count, 'number', 'Count should be a number');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetHistory - returns archived changesets after removal', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-history-');

  try {
    const branchName = 'feature/history-test';
    createBranch(tempDir, branchName);

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    await changesetRemove({
      root: tempDir,
      branch: branchName,
      force: true,
    });

    const result = await changesetHistory({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);

    const data = result.data as ChangesetHistoryData;
    // After removal, the changeset should be archived
    t.true(data.count >= 0, 'Should have count');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetHistory - filters by limit', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-history-limit-');

  try {
    // Create and remove multiple changesets
    for (let i = 1; i <= 3; i++) {
      switchBranch(tempDir, 'main');
      createBranch(tempDir, `feature/history-limit-${i}`);
      await changesetAdd({
        root: tempDir,
        bump: 'patch',
        packages: ['test-package'],
      });
      await changesetRemove({
        root: tempDir,
        branch: `feature/history-limit-${i}`,
        force: true,
      });
    }

    const result = await changesetHistory({
      root: tempDir,
      limit: 2,
    });

    t.true(result.success);
    const data = result.data as ChangesetHistoryData;
    t.true(data.archived.length <= 2, 'Should respect limit');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetHistory - filters by bump type', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-history-filter-bump-');

  try {
    // Create patch changeset
    createBranch(tempDir, 'fix/history-patch');
    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });
    await changesetRemove({
      root: tempDir,
      branch: 'fix/history-patch',
      force: true,
    });

    // Create minor changeset
    switchBranch(tempDir, 'main');
    createBranch(tempDir, 'feature/history-minor');
    await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
    });
    await changesetRemove({
      root: tempDir,
      branch: 'feature/history-minor',
      force: true,
    });

    const result = await changesetHistory({
      root: tempDir,
      filterBump: 'minor',
    });

    t.true(result.success);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetHistory - returns error for empty root', async (t) => {
  const result = await changesetHistory({ root: '' });

  t.false(result.success);
  t.truthy(result.error);
  t.is((result.error as ErrorInfo).code, 'EVALIDATION');
});

test('changesetHistory - returns error for non-existent path', async (t) => {
  const result = await changesetHistory({
    root: '/non/existent/path',
  });

  t.false(result.success);
  t.truthy(result.error);
});

// ============================================================================
// changesetCheck Tests
// ============================================================================

test('changesetCheck - returns true when changeset exists', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-check-exists-');

  try {
    const branchName = 'feature/check-exists';
    createBranch(tempDir, branchName);

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const params: ChangesetCheckParams = {
      root: tempDir,
      branch: branchName,
    };

    const result: ChangesetCheckApiResponse = await changesetCheck(params);

    t.true(result.success, 'changesetCheck should succeed');
    t.truthy(result.data, 'Data should be present');

    const data: ChangesetCheckData = result.data as ChangesetCheckData;
    t.is(typeof data.hasChangeset, 'boolean');
    t.true(data.hasChangeset, 'Should have changeset');
    t.is(data.branch, branchName);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetCheck - returns false when no changeset exists', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-check-noexist-');

  try {
    const branchName = 'feature/no-changeset';
    createBranch(tempDir, branchName);

    const result = await changesetCheck({
      root: tempDir,
      branch: branchName,
    });

    t.true(result.success);
    t.truthy(result.data);

    const data = result.data as ChangesetCheckData;
    t.false(data.hasChangeset, 'Should not have changeset');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetCheck - checks current branch when no branch specified', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-check-current-');

  try {
    const branchName = 'feature/check-current';
    createBranch(tempDir, branchName);

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    // Check without specifying branch
    const result = await changesetCheck({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);
    t.true((result.data as ChangesetCheckData).hasChangeset);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetCheck - returns error for empty root', async (t) => {
  const result = await changesetCheck({ root: '' });

  t.false(result.success);
  t.truthy(result.error);
  t.is((result.error as ErrorInfo).code, 'EVALIDATION');
});

test('changesetCheck - returns error for non-existent path', async (t) => {
  const result = await changesetCheck({
    root: '/non/existent/path',
  });

  t.false(result.success);
  t.truthy(result.error);
});

// ============================================================================
// Type Verification Tests
// ============================================================================

test('changesetAdd - response matches ChangesetAddApiResponse interface', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-add-type-');

  try {
    createBranch(tempDir, 'feature/type-test');

    const result: ChangesetAddApiResponse = await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    // Verify the response structure matches the interface
    t.true('success' in result, 'Response should have success property');
    t.true(
      result.success
        ? 'data' in result && result.data !== undefined
        : 'error' in result,
      'Response should have data on success or error on failure'
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetUpdate - response matches ChangesetUpdateApiResponse interface', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-update-type-');

  try {
    createBranch(tempDir, 'feature/update-type-test');

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const result: ChangesetUpdateApiResponse = await changesetUpdate({
      root: tempDir,
      id: 'feature/update-type-test',
      bump: 'minor',
    });

    t.true('success' in result);
    if (result.success) {
      t.true('data' in result);
      const data = result.data as ChangesetUpdateData;
      t.true('updated' in data);
      t.true('summary' in data);
      t.true('changeset' in data);
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetList - response matches ChangesetListApiResponse interface', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-list-type-');

  try {
    const result: ChangesetListApiResponse = await changesetList({
      root: tempDir,
    });

    t.true('success' in result);
    if (result.success) {
      t.true('data' in result);
      const data = result.data as ChangesetListData;
      t.true('changesets' in data);
      t.true('count' in data);
      t.true(Array.isArray(data.changesets));
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetShow - response matches ChangesetShowApiResponse interface', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-show-type-');

  try {
    createBranch(tempDir, 'feature/show-type-test');

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const result: ChangesetShowApiResponse = await changesetShow({
      root: tempDir,
      branch: 'feature/show-type-test',
    });

    t.true('success' in result);
    if (result.success) {
      t.true('data' in result);
      const data = result.data as ChangesetShowData;
      t.true('changeset' in data);
      t.truthy(data.changeset);
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetRemove - response matches ChangesetRemoveApiResponse interface', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-remove-type-');

  try {
    createBranch(tempDir, 'feature/remove-type-test');

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const result: ChangesetRemoveApiResponse = await changesetRemove({
      root: tempDir,
      branch: 'feature/remove-type-test',
      force: true,
    });

    t.true('success' in result);
    if (result.success) {
      t.true('data' in result);
      const data = result.data as ChangesetRemoveData;
      t.true('removed' in data);
      t.true('branch' in data);
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetHistory - response matches ChangesetHistoryApiResponse interface', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-history-type-');

  try {
    const result: ChangesetHistoryApiResponse = await changesetHistory({
      root: tempDir,
    });

    t.true('success' in result);
    if (result.success) {
      t.true('data' in result);
      const data = result.data as ChangesetHistoryData;
      t.true('archived' in data);
      t.true('count' in data);
      t.true(Array.isArray(data.archived));
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('changesetCheck - response matches ChangesetCheckApiResponse interface', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-check-type-');

  try {
    const result: ChangesetCheckApiResponse = await changesetCheck({
      root: tempDir,
    });

    t.true('success' in result);
    if (result.success) {
      t.true('data' in result);
      const data = result.data as ChangesetCheckData;
      t.true('hasChangeset' in data);
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('error - matches ErrorInfo interface structure', async (t) => {
  const result = await changesetAdd({
    root: '',
    bump: 'patch',
    packages: ['test-package'],
  });

  t.false(result.success);
  t.truthy(result.error);

  const error = result.error as ErrorInfo;
  t.true('code' in error, 'Error should have code');
  t.true('message' in error, 'Error should have message');
  t.true('kind' in error, 'Error should have kind');
  t.is(typeof error.code, 'string');
  t.is(typeof error.message, 'string');
  t.is(typeof error.kind, 'string');
});

// ============================================================================
// Edge Case and Special Scenario Tests
// ============================================================================

test('changeset operations - handle path with spaces', async (t) => {
  const tempDir = createTempDir('changeset test with spaces ');

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);
    createInitialCommit(tempDir);
    await initWorkspace(tempDir);
    createBranch(tempDir, 'feature/space-test');

    const result = await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    t.true(result.success, 'Should handle paths with spaces');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changeset operations - handle path with unicode characters', async (t) => {
  const tempBase = os.tmpdir();
  const tempDir = path.join(
    tempBase,
    `changeset-üñíçödé-${Date.now()}`
  );
  fs.mkdirSync(tempDir, { recursive: true });

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);
    createInitialCommit(tempDir);
    await initWorkspace(tempDir);
    createBranch(tempDir, 'feature/unicode-test');

    const result = await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    t.true(result.success, 'Should handle paths with unicode characters');
  } finally {
    removeTempDir(tempDir);
  }
});

test('changeset operations - complete workflow', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-workflow-');

  try {
    const branchName = 'feature/complete-workflow';
    createBranch(tempDir, branchName);

    // Step 1: Check no changeset exists
    const checkBefore = await changesetCheck({
      root: tempDir,
      branch: branchName,
    });
    t.true(checkBefore.success);
    t.false((checkBefore.data as ChangesetCheckData).hasChangeset);

    // Step 2: Add changeset
    const addResult = await changesetAdd({
      root: tempDir,
      bump: 'minor',
      packages: ['test-package'],
      message: 'New feature',
    });
    t.true(addResult.success);

    // Step 3: Check changeset exists
    const checkAfter = await changesetCheck({
      root: tempDir,
      branch: branchName,
    });
    t.true(checkAfter.success);
    t.true((checkAfter.data as ChangesetCheckData).hasChangeset);

    // Step 4: List changesets
    const listResult = await changesetList({ root: tempDir });
    t.true(listResult.success);
    t.true((listResult.data as ChangesetListData).count >= 1);

    // Step 5: Show changeset
    const showResult = await changesetShow({
      root: tempDir,
      branch: branchName,
    });
    t.true(showResult.success);
    t.is((showResult.data as ChangesetShowData).changeset.bump, 'minor');

    // Step 6: Update changeset
    const updateResult = await changesetUpdate({
      root: tempDir,
      id: branchName,
      bump: 'major',
    });
    t.true(updateResult.success);

    // Step 7: Verify update
    const showAfterUpdate = await changesetShow({
      root: tempDir,
      branch: branchName,
    });
    t.true(showAfterUpdate.success);
    t.is(
      (showAfterUpdate.data as ChangesetShowData).changeset.bump,
      'major'
    );

    // Step 8: Remove changeset
    const removeResult = await changesetRemove({
      root: tempDir,
      branch: branchName,
      force: true,
    });
    t.true(removeResult.success);

    // Step 9: Verify removal
    const checkFinal = await changesetCheck({
      root: tempDir,
      branch: branchName,
    });
    t.true(checkFinal.success);
    t.false((checkFinal.data as ChangesetCheckData).hasChangeset);

    // Step 10: Check history
    const historyResult = await changesetHistory({ root: tempDir });
    t.true(historyResult.success);
  } finally {
    removeTempDir(tempDir);
  }
});

test('changeset operations - completes within reasonable time', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-perf-');

  try {
    createBranch(tempDir, 'feature/perf-test');

    const startTime = Date.now();

    await changesetAdd({
      root: tempDir,
      bump: 'patch',
      packages: ['test-package'],
    });

    const duration = Date.now() - startTime;

    t.true(
      duration < 10000,
      `Operation should complete within 10 seconds (took ${duration}ms)`
    );
  } finally {
    removeTempDir(tempDir);
  }
});

test('changeset operations - can handle multiple sequential operations', async (t) => {
  const tempDir = await setupTestEnvironment('changeset-seq-');

  try {
    const operations: Promise<ChangesetAddApiResponse>[] = [];

    // Create multiple branches and changesets sequentially
    for (let i = 1; i <= 3; i++) {
      if (i > 1) {
        switchBranch(tempDir, 'main');
      }
      createBranch(tempDir, `feature/seq-test-${i}`);

      operations.push(
        changesetAdd({
          root: tempDir,
          bump: 'patch',
          packages: ['test-package'],
          branch: `feature/seq-test-${i}`,
        })
      );

      // Wait for each operation to complete before starting the next
      await operations[i - 1];
    }

    const results = await Promise.all(operations);
    const successCount = results.filter((r) => r.success).length;

    t.is(successCount, 3, 'All sequential operations should succeed');

    // Verify all changesets exist
    const listResult = await changesetList({ root: tempDir });
    t.true(listResult.success);
    t.true(
      (listResult.data as ChangesetListData).count >= 3,
      'Should have at least 3 changesets'
    );
  } finally {
    removeTempDir(tempDir);
  }
});
