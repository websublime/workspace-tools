/**
 * Integration tests for the config commands (configShow and configValidate).
 *
 * ## What
 *
 * This module contains comprehensive integration tests for the config NAPI functions:
 * - `configShow`: Loads and displays the workspace configuration
 * - `configValidate`: Validates the workspace configuration and reports issues
 *
 * ## How
 *
 * Tests are organized into logical groups:
 * - configShow tests: Verify configuration loading and display functionality
 * - configValidate tests: Verify configuration validation and error reporting
 * - Error tests: Verify proper error handling for invalid inputs
 * - Type verification tests: Ensure TypeScript types match actual response structure
 *
 * Each test creates an isolated temporary directory with:
 * - A package.json for workspace identification
 * - An initialized git repository with proper configuration
 * - Configuration files in various formats (JSON, YAML, TOML)
 *
 * ## Why
 *
 * Integration tests validate that the Node.js bindings work correctly end-to-end,
 * ensuring the Rust code, NAPI bindings, and TypeScript types are all aligned.
 * The config commands are essential for introspecting and validating workspace
 * configuration, enabling tooling and CI/CD integration.
 *
 * @packageDocumentation
 */

import test from 'ava';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';

import { configShow, configValidate, init } from '../src/index';
import type {
  ConfigShowParams,
  ConfigShowApiResponse,
  ConfigShowData,
  ConfigValidateParams,
  ConfigValidateApiResponse,
  ConfigValidateData,
  ConfigData,
  ConfigValidationIssue,
  ChangesetConfigInfo,
  VersionConfigInfo,
  DependencyConfigInfo,
  UpgradeConfigInfo,
  ChangelogConfigInfo,
  AuditConfigInfo,
  GitConfigInfo,
  ExecuteConfigInfo,
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
  const currentDir = process.cwd();

  // If we're in the workspace root, it should have the root pnpm-workspace.yaml
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
function createTempDir(prefix: string = 'config-test-'): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

/**
 * Removes a directory and all its contents recursively.
 * Includes retry logic for Windows where files may be locked temporarily.
 */
function removeTempDir(dirPath: string): void {
  if (!fs.existsSync(dirPath)) {
    return;
  }

  const maxRetries = 3;
  const retryDelayMs = 500;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      fs.rmSync(dirPath, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
      return;
    } catch (error: unknown) {
      const isLastAttempt = attempt === maxRetries;
      const isPermissionError =
        (error as NodeJS.ErrnoException).code === 'EPERM' ||
        (error as NodeJS.ErrnoException).code === 'EBUSY';

      if (isLastAttempt || !isPermissionError) {
        // On final attempt or non-permission error, log warning but don't throw
        console.warn(`[cleanup] Failed to remove ${dirPath}: ${(error as Error).message}`);
        return;
      }

      // Wait before retry (synchronous delay for simplicity in cleanup)
      const start = Date.now();
      while (Date.now() - start < retryDelayMs) {
        // Busy wait - acceptable for test cleanup
      }
    }
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
 * Creates a valid repo.config.json file in the given directory.
 */
function createConfigJson(dirPath: string, config: object = {}): void {
  const defaultConfig = {
    changeset: {
      path: '.changesets',
      history_path: '.changesets/history',
      available_environments: ['development', 'staging', 'production'],
      default_environments: ['development'],
    },
    version: {
      strategy: 'independent',
      default_bump: 'patch',
    },
    dependency: {
      propagate_dependencies: true,
      propagate_dev_dependencies: false,
      propagate_peer_dependencies: false,
      propagation_bump: 'patch',
      max_depth: 5,
      fail_on_circular: true,
    },
    upgrade: {
      auto_changeset: false,
      changeset_bump: 'minor',
      backup: {
        enabled: true,
        path: '.backups',
        keep_count: 5,
      },
    },
    changelog: {
      enabled: true,
      format: 'keep-a-changelog',
      include_commit_links: false,
    },
    audit: {
      enabled: true,
      min_severity: 'info',
    },
    git: {
      branch_base: 'main',
      detect_affected_packages: true,
    },
    execute: {
      timeout_secs: 300,
      per_package_timeout_secs: 60,
      max_parallel: 4,
    },
  };

  const mergedConfig = { ...defaultConfig, ...config };
  fs.writeFileSync(
    path.join(dirPath, 'repo.config.json'),
    JSON.stringify(mergedConfig, null, 2)
  );
}

/**
 * Creates a valid repo.config.yaml file in the given directory.
 */
function createConfigYaml(dirPath: string): void {
  const yamlContent = `changeset:
  path: ".changesets"
  history_path: ".changesets/history"
  available_environments:
    - development
    - staging
    - production
  default_environments:
    - development

version:
  strategy: independent
  default_bump: patch

dependency:
  propagate_dependencies: true
  propagate_dev_dependencies: false
  propagate_peer_dependencies: false
  propagation_bump: patch
  max_depth: 5
  fail_on_circular: true

upgrade:
  auto_changeset: false
  changeset_bump: minor
  backup:
    enabled: true
    path: ".backups"
    keep_count: 5

changelog:
  enabled: true
  format: keep-a-changelog
  include_commit_links: false

audit:
  enabled: true
  min_severity: info

git:
  branch_base: main
  detect_affected_packages: true

execute:
  timeout_secs: 300
  per_package_timeout_secs: 60
  max_parallel: 4
`;

  fs.writeFileSync(path.join(dirPath, 'repo.config.yaml'), yamlContent);
}

/**
 * Creates a valid repo.config.toml file in the given directory.
 */
function createConfigToml(dirPath: string): void {
  const tomlContent = `[changeset]
path = ".changesets"
history_path = ".changesets/history"
available_environments = ["development", "staging", "production"]
default_environments = ["development"]

[version]
strategy = "independent"
default_bump = "patch"

[dependency]
propagate_dependencies = true
propagate_dev_dependencies = false
propagate_peer_dependencies = false
propagation_bump = "patch"
max_depth = 5
fail_on_circular = true

[upgrade]
auto_changeset = false
changeset_bump = "minor"

[upgrade.backup]
enabled = true
path = ".backups"
keep_count = 5

[changelog]
enabled = true
format = "keep-a-changelog"
include_commit_links = false

[audit]
enabled = true
min_severity = "info"

[git]
branch_base = "main"
detect_affected_packages = true

[execute]
timeout_secs = 300
per_package_timeout_secs = 60
max_parallel = 4
`;

  fs.writeFileSync(path.join(dirPath, 'repo.config.toml'), tomlContent);
}

/**
 * Creates an invalid repo.config.json file with structural errors.
 */
function createInvalidConfigJson(dirPath: string): void {
  const invalidConfig = {
    changeset: {
      path: '.changesets',
    },
    version: {
      strategy: 'invalid-strategy', // Invalid strategy value
      default_bump: 'patch',
    },
  };

  fs.writeFileSync(
    path.join(dirPath, 'repo.config.json'),
    JSON.stringify(invalidConfig, null, 2)
  );
}

/**
 * Creates a config with semantic warnings (valid but with potential issues).
 */
function createConfigWithWarnings(dirPath: string): void {
  const configWithWarnings = {
    changeset: {
      path: '.changesets',
      history_path: '.changesets/history',
      available_environments: ['development'],
      default_environments: ['development'],
    },
    version: {
      strategy: 'independent',
      default_bump: 'patch',
    },
    dependency: {
      propagate_dependencies: true,
      propagate_dev_dependencies: false,
      propagate_peer_dependencies: false,
      propagation_bump: 'patch',
      max_depth: 15, // High depth may cause performance issues - warning
      fail_on_circular: true,
    },
    upgrade: {
      auto_changeset: false,
      changeset_bump: 'minor',
      backup: {
        enabled: true,
        path: '.backups',
        keep_count: 5,
      },
    },
    changelog: {
      enabled: true,
      format: 'keep-a-changelog',
      include_commit_links: true, // Commit links enabled but no repository_url - warning
    },
    audit: {
      enabled: true,
      min_severity: 'info',
    },
    git: {
      branch_base: 'main',
      detect_affected_packages: true,
    },
    execute: {
      timeout_secs: 5, // Very short timeout - warning
      per_package_timeout_secs: 3, // Very short timeout - warning
      max_parallel: 32, // High parallelism - warning
    },
  };

  fs.writeFileSync(
    path.join(dirPath, 'repo.config.json'),
    JSON.stringify(configWithWarnings, null, 2)
  );
}

/**
 * Sets up a complete workspace for testing.
 * Creates package.json, git repo, and optionally a config file.
 */
function setupWorkspace(
  tempDir: string,
  options: {
    createConfig?: boolean;
    configFormat?: 'json' | 'yaml' | 'toml';
    configOverrides?: object;
    invalidConfig?: boolean;
    configWithWarnings?: boolean;
  } = {}
): void {
  createPackageJson(tempDir);
  initGitRepo(tempDir);

  if (options.invalidConfig) {
    createInvalidConfigJson(tempDir);
  } else if (options.configWithWarnings) {
    createConfigWithWarnings(tempDir);
  } else if (options.createConfig !== false) {
    const format = options.configFormat || 'json';
    if (format === 'json') {
      createConfigJson(tempDir, options.configOverrides || {});
    } else if (format === 'yaml') {
      createConfigYaml(tempDir);
    } else if (format === 'toml') {
      createConfigToml(tempDir);
    }
  }
}

// ============================================================================
// configShow Success Tests
// ============================================================================

test('configShow - returns configuration with proper structure', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const params: ConfigShowParams = {
      root: tempDir,
    };

    const result: ConfigShowApiResponse = await configShow(params);

    // Verify success
    t.true(result.success, 'configShow command should succeed for valid workspace');
    t.truthy(result.data, 'Data should be present on success');
    t.is(result.error, undefined, 'Error should not be present on success');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns config path and format', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'json' });

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);

    const data: ConfigShowData = result.data as ConfigShowData;

    // Verify config path and format
    t.is(typeof data.configPath, 'string', 'Config path should be a string');
    t.true(data.configPath.includes('repo.config'), 'Config path should contain repo.config');
    t.is(data.configFormat, 'json', 'Config format should be json');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - reads JSON configuration correctly', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'json' });

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);

    const data: ConfigShowData = result.data as ConfigShowData;
    t.is(data.configFormat, 'json');
    t.truthy(data.config, 'Config object should be present');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - reads YAML configuration correctly', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'yaml' });

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);

    const data: ConfigShowData = result.data as ConfigShowData;
    t.is(data.configFormat, 'yaml');
    t.truthy(data.config, 'Config object should be present');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - reads TOML configuration correctly', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'toml' });

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    t.truthy(result.data);

    const data: ConfigShowData = result.data as ConfigShowData;
    t.is(data.configFormat, 'toml');
    t.truthy(data.config, 'Config object should be present');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns changeset configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const changeset: ChangesetConfigInfo = data.config.changeset;

    t.is(typeof changeset.path, 'string', 'Changeset path should be a string');
    t.is(typeof changeset.historyPath, 'string', 'History path should be a string');
    t.true(Array.isArray(changeset.availableEnvironments), 'Available environments should be an array');
    t.true(Array.isArray(changeset.defaultEnvironments), 'Default environments should be an array');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns version configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const version: VersionConfigInfo = data.config.version;

    t.is(typeof version.strategy, 'string', 'Strategy should be a string');
    t.true(
      ['independent', 'unified'].includes(version.strategy),
      'Strategy should be independent or unified'
    );
    t.is(typeof version.defaultBump, 'string', 'Default bump should be a string');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns dependency configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const dependency: DependencyConfigInfo = data.config.dependency;

    t.is(typeof dependency.propagateDependencies, 'boolean');
    t.is(typeof dependency.propagateDevDependencies, 'boolean');
    t.is(typeof dependency.propagatePeerDependencies, 'boolean');
    t.is(typeof dependency.propagationBump, 'string');
    t.is(typeof dependency.maxDepth, 'number');
    t.is(typeof dependency.failOnCircular, 'boolean');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns upgrade configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const upgrade: UpgradeConfigInfo = data.config.upgrade;

    t.is(typeof upgrade.autoChangeset, 'boolean');
    t.is(typeof upgrade.changesetBump, 'string');
    t.truthy(upgrade.backup, 'Backup config should be present');
    t.is(typeof upgrade.backup.enabled, 'boolean');
    t.is(typeof upgrade.backup.path, 'string');
    t.is(typeof upgrade.backup.keepCount, 'number');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns changelog configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const changelog: ChangelogConfigInfo = data.config.changelog;

    t.is(typeof changelog.enabled, 'boolean');
    t.is(typeof changelog.format, 'string');
    t.is(typeof changelog.includeCommitLinks, 'boolean');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns audit configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const audit: AuditConfigInfo = data.config.audit;

    t.is(typeof audit.enabled, 'boolean');
    t.is(typeof audit.minSeverity, 'string');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns git configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const git: GitConfigInfo = data.config.git;

    t.is(typeof git.branchBase, 'string');
    t.is(typeof git.detectAffectedPackages, 'boolean');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns execute configuration section', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success);
    const data: ConfigShowData = result.data as ConfigShowData;
    const execute: ExecuteConfigInfo = data.config.execute;

    t.is(typeof execute.timeoutSecs, 'number');
    t.is(typeof execute.perPackageTimeoutSecs, 'number');
    t.is(typeof execute.maxParallel, 'number');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - works with real workspace configuration', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  // Only run this test if we have a real workspace with config
  const configExists =
    fs.existsSync(path.join(workspaceRoot, 'repo.config.json')) ||
    fs.existsSync(path.join(workspaceRoot, 'repo.config.yaml')) ||
    fs.existsSync(path.join(workspaceRoot, 'repo.config.toml'));

  if (!configExists) {
    t.pass('Skipping test - no workspace config found');
    return;
  }

  const result = await configShow({ root: workspaceRoot });

  t.true(result.success, 'Should load real workspace configuration');
  t.truthy(result.data);
  t.truthy(result.data?.config);
});

// ============================================================================
// configShow Error Tests
// ============================================================================

test('configShow - returns error for non-existent path', async (t) => {
  const nonExistentPath = '/non/existent/path/that/does/not/exist';

  const result = await configShow({ root: nonExistentPath });

  t.false(result.success, 'Should fail for non-existent path');
  t.is(result.data, undefined, 'Data should not be present on error');
  t.truthy(result.error, 'Error should be present');

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(error.code, 'ENOENT', 'Error code should be ENOENT');
  t.is(typeof error.message, 'string');
  t.true(error.message.length > 0, 'Error message should not be empty');
});

test('configShow - returns error for empty root path', async (t) => {
  const result = await configShow({ root: '' });

  t.false(result.success, 'Should fail for empty root path');
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
});

test('configShow - returns error for file path instead of directory', async (t) => {
  const tempDir = createTempDir();
  const tempFile = path.join(tempDir, 'test_file.txt');

  try {
    fs.writeFileSync(tempFile, 'test content');

    const result = await configShow({ root: tempFile });

    t.false(result.success, 'Should fail for file path');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - returns error when no config file exists', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { createConfig: false });

    const result = await configShow({ root: tempDir });

    t.false(result.success, 'Should fail when no config file exists');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'ECONFIG', 'Error code should be ECONFIG');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - error contains helpful context', async (t) => {
  const result = await configShow({ root: '/nonexistent' });

  t.false(result.success);
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(typeof error.code, 'string');
  t.is(typeof error.message, 'string');
  t.true(error.message.length > 0);
});

// ============================================================================
// configValidate Success Tests
// ============================================================================

test('configValidate - returns valid for correct configuration', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const params: ConfigValidateParams = {
      root: tempDir,
    };

    const result: ConfigValidateApiResponse = await configValidate(params);

    t.true(result.success, 'configValidate command should succeed');
    t.truthy(result.data, 'Data should be present on success');
    t.is(result.error, undefined, 'Error should not be present on success');

    const data: ConfigValidateData = result.data as ConfigValidateData;
    t.true(data.valid, 'Configuration should be valid');
    t.is(typeof data.configPath, 'string');
    t.true(Array.isArray(data.errors), 'Errors should be an array');
    t.true(Array.isArray(data.warnings), 'Warnings should be an array');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - returns config path in response', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'json' });

    const result = await configValidate({ root: tempDir });

    t.true(result.success);
    const data: ConfigValidateData = result.data as ConfigValidateData;

    t.is(typeof data.configPath, 'string');
    t.true(data.configPath.includes('repo.config'), 'Config path should contain repo.config');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - valid config has empty errors array', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configValidate({ root: tempDir });

    t.true(result.success);
    const data: ConfigValidateData = result.data as ConfigValidateData;

    t.true(data.valid);
    t.is(data.errors.length, 0, 'Valid config should have no errors');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - validates JSON configuration', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'json' });

    const result = await configValidate({ root: tempDir });

    t.true(result.success);
    const data: ConfigValidateData = result.data as ConfigValidateData;
    t.true(data.valid);
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - validates YAML configuration', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'yaml' });

    const result = await configValidate({ root: tempDir });

    t.true(result.success);
    const data: ConfigValidateData = result.data as ConfigValidateData;
    t.true(data.valid);
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - validates TOML configuration', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configFormat: 'toml' });

    const result = await configValidate({ root: tempDir });

    t.true(result.success);
    const data: ConfigValidateData = result.data as ConfigValidateData;
    t.true(data.valid);
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - returns warnings for configuration with issues', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configWithWarnings: true });

    const result = await configValidate({ root: tempDir });

    t.true(result.success, 'Command should succeed even with warnings');
    const data: ConfigValidateData = result.data as ConfigValidateData;

    // Config with warnings should still be valid (warnings are not errors)
    t.true(data.valid, 'Config should be valid despite warnings');
    t.true(data.warnings.length > 0, 'Should have warnings for semantic issues');

    // Check warning structure
    const warning = data.warnings[0];
    t.is(typeof warning.severity, 'string');
    t.is(typeof warning.field, 'string');
    t.is(typeof warning.message, 'string');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - warning has proper structure', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { configWithWarnings: true });

    const result = await configValidate({ root: tempDir });

    t.true(result.success);
    const data: ConfigValidateData = result.data as ConfigValidateData;

    if (data.warnings.length > 0) {
      const warning: ConfigValidationIssue = data.warnings[0];
      t.is(warning.severity, 'warning', 'Warning severity should be "warning"');
      t.true(warning.field.length > 0, 'Field should not be empty');
      t.true(warning.message.length > 0, 'Message should not be empty');
    } else {
      t.pass('No warnings generated (acceptable)');
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - works with real workspace configuration', async (t) => {
  const workspaceRoot = getWorkspaceRoot();

  // Only run this test if we have a real workspace with config
  const configExists =
    fs.existsSync(path.join(workspaceRoot, 'repo.config.json')) ||
    fs.existsSync(path.join(workspaceRoot, 'repo.config.yaml')) ||
    fs.existsSync(path.join(workspaceRoot, 'repo.config.toml'));

  if (!configExists) {
    t.pass('Skipping test - no workspace config found');
    return;
  }

  const result = await configValidate({ root: workspaceRoot });

  t.true(result.success, 'Should validate real workspace configuration');
  t.truthy(result.data);
});

// ============================================================================
// configValidate Error Tests
// ============================================================================

test('configValidate - returns error for non-existent path', async (t) => {
  const nonExistentPath = '/non/existent/path/that/does/not/exist';

  const result = await configValidate({ root: nonExistentPath });

  t.false(result.success, 'Should fail for non-existent path');
  t.is(result.data, undefined, 'Data should not be present on error');
  t.truthy(result.error, 'Error should be present');

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(error.code, 'ENOENT', 'Error code should be ENOENT');
});

test('configValidate - returns error for empty root path', async (t) => {
  const result = await configValidate({ root: '' });

  t.false(result.success, 'Should fail for empty root path');
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;
  t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
});

test('configValidate - returns error for file path instead of directory', async (t) => {
  const tempDir = createTempDir();
  const tempFile = path.join(tempDir, 'test_file.txt');

  try {
    fs.writeFileSync(tempFile, 'test content');

    const result = await configValidate({ root: tempFile });

    t.false(result.success, 'Should fail for file path');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'EVALIDATION', 'Error code should be EVALIDATION');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - returns error when no config file exists', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir, { createConfig: false });

    const result = await configValidate({ root: tempDir });

    t.false(result.success, 'Should fail when no config file exists');
    t.truthy(result.error);

    const error: ErrorInfo = result.error as ErrorInfo;
    t.is(error.code, 'ECONFIG', 'Error code should be ECONFIG');
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Type Verification Tests
// ============================================================================

test('configShow - response matches ConfigShowApiResponse interface', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result: ConfigShowApiResponse = await configShow({ root: tempDir });

    // Verify interface compliance
    t.is(typeof result.success, 'boolean');
    t.true(result.data === undefined || typeof result.data === 'object');
    t.true(result.error === undefined || typeof result.error === 'object');

    // On success, verify data structure
    if (result.success && result.data) {
      t.is(typeof result.data.configPath, 'string');
      t.is(typeof result.data.configFormat, 'string');
      t.is(typeof result.data.config, 'object');
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - data matches ConfigShowData interface structure', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });
    t.true(result.success);

    const data: ConfigShowData = result.data as ConfigShowData;

    // Verify all required fields are present
    t.is(typeof data.configPath, 'string');
    t.is(typeof data.configFormat, 'string');
    t.truthy(data.config);

    // Verify config structure
    const config: ConfigData = data.config;
    t.truthy(config.changeset);
    t.truthy(config.version);
    t.truthy(config.dependency);
    t.truthy(config.upgrade);
    t.truthy(config.changelog);
    t.truthy(config.audit);
    t.truthy(config.git);
    t.truthy(config.execute);
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - response matches ConfigValidateApiResponse interface', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result: ConfigValidateApiResponse = await configValidate({ root: tempDir });

    // Verify interface compliance
    t.is(typeof result.success, 'boolean');
    t.true(result.data === undefined || typeof result.data === 'object');
    t.true(result.error === undefined || typeof result.error === 'object');

    // On success, verify data structure
    if (result.success && result.data) {
      t.is(typeof result.data.valid, 'boolean');
      t.is(typeof result.data.configPath, 'string');
      t.true(Array.isArray(result.data.errors));
      t.true(Array.isArray(result.data.warnings));
    }
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - data matches ConfigValidateData interface structure', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const result = await configValidate({ root: tempDir });
    t.true(result.success);

    const data: ConfigValidateData = result.data as ConfigValidateData;

    // Verify all required fields are present and typed correctly
    t.is(typeof data.valid, 'boolean');
    t.is(typeof data.configPath, 'string');
    t.true(Array.isArray(data.errors));
    t.true(Array.isArray(data.warnings));
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - ErrorInfo structure on failure', async (t) => {
  const result = await configShow({ root: '/nonexistent' });

  t.false(result.success, 'Should fail for non-existent path');
  t.truthy(result.error, 'Error should be present');

  const error: ErrorInfo = result.error as ErrorInfo;

  // Verify ErrorInfo structure
  t.is(typeof error.code, 'string');
  t.is(typeof error.message, 'string');
  t.true(error.context === undefined || typeof error.context === 'string' || typeof error.context === 'object');
  t.true(error.kind === undefined || typeof error.kind === 'string');
});

test('configValidate - error matches ErrorInfo interface structure', async (t) => {
  const result = await configValidate({ root: '/nonexistent' });

  t.false(result.success);
  t.truthy(result.error);

  const error: ErrorInfo = result.error as ErrorInfo;

  // Verify ErrorInfo structure
  t.is(typeof error.code, 'string');
  t.is(typeof error.message, 'string');
});

// ============================================================================
// Edge Case Tests
// ============================================================================

test('configShow - handles path with spaces', async (t) => {
  const tempBase = createTempDir('config test with spaces ');
  const tempDir = tempBase;

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success, 'Should handle paths with spaces');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - handles path with unicode characters', async (t) => {
  const tempBase = os.tmpdir();
  const tempDir = fs.mkdtempSync(path.join(tempBase, 'config-テスト-'));

  try {
    setupWorkspace(tempDir);

    const result = await configShow({ root: tempDir });

    t.true(result.success, 'Should handle paths with unicode characters');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - handles absolute path', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const absolutePath = path.resolve(tempDir);
    const result = await configShow({ root: absolutePath });

    t.true(result.success, 'Should handle absolute paths');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - handles path with spaces', async (t) => {
  const tempBase = createTempDir('validate test with spaces ');
  const tempDir = tempBase;

  try {
    setupWorkspace(tempDir);

    const result = await configValidate({ root: tempDir });

    t.true(result.success, 'Should handle paths with spaces');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - handles absolute path', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const absolutePath = path.resolve(tempDir);
    const result = await configValidate({ root: absolutePath });

    t.true(result.success, 'Should handle absolute paths');
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Performance Tests
// ============================================================================

test('configShow - completes within reasonable time', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const startTime = Date.now();
    await configShow({ root: tempDir });
    const duration = Date.now() - startTime;

    t.true(duration < 5000, `configShow should complete within 5 seconds, took ${duration}ms`);
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - completes within reasonable time', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const startTime = Date.now();
    await configValidate({ root: tempDir });
    const duration = Date.now() - startTime;

    t.true(duration < 5000, `configValidate should complete within 5 seconds, took ${duration}ms`);
  } finally {
    removeTempDir(tempDir);
  }
});

test('configShow - can be called multiple times', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const results = await Promise.all([
      configShow({ root: tempDir }),
      configShow({ root: tempDir }),
      configShow({ root: tempDir }),
    ]);

    t.true(results.every((r) => r.success), 'All calls should succeed');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - can be called multiple times', async (t) => {
  const tempDir = createTempDir();

  try {
    setupWorkspace(tempDir);

    const results = await Promise.all([
      configValidate({ root: tempDir }),
      configValidate({ root: tempDir }),
      configValidate({ root: tempDir }),
    ]);

    t.true(results.every((r) => r.success), 'All calls should succeed');
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Integration with init command
// ============================================================================

test('configShow - reads config created by init command', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    // Use init to create config
    const initResult = await init({
      root: tempDir,
      strategy: 'independent',
      configFormat: 'json',
    });

    t.true(initResult.success, 'Init should succeed');

    // Now use configShow to read it
    const showResult = await configShow({ root: tempDir });

    t.true(showResult.success, 'configShow should read init-created config');
    t.truthy(showResult.data);

    const data: ConfigShowData = showResult.data as ConfigShowData;
    t.is(data.config.version.strategy, 'independent', 'Strategy should match init params');
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - validates config created by init command', async (t) => {
  const tempDir = createTempDir();

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    // Use init to create config
    const initResult = await init({
      root: tempDir,
      strategy: 'unified',
      configFormat: 'yaml',
    });

    t.true(initResult.success, 'Init should succeed');

    // Now use configValidate to validate it
    const validateResult = await configValidate({ root: tempDir });

    t.true(validateResult.success, 'configValidate should succeed');
    t.truthy(validateResult.data);

    const data: ConfigValidateData = validateResult.data as ConfigValidateData;
    t.true(data.valid, 'Config created by init should be valid');
    t.is(data.errors.length, 0, 'Should have no errors');
  } finally {
    removeTempDir(tempDir);
  }
});

// ============================================================================
// Custom Config Path Tests
// ============================================================================

test('configShow - accepts custom config path', async (t) => {
  const tempDir = createTempDir();
  const customConfigDir = path.join(tempDir, 'custom');

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    // Create config in custom location
    fs.mkdirSync(customConfigDir, { recursive: true });
    createConfigJson(customConfigDir);

    const customConfigPath = path.join(customConfigDir, 'repo.config.json');

    const result = await configShow({
      root: tempDir,
      configPath: customConfigPath,
    });

    t.true(result.success, 'Should read config from custom path');
    t.truthy(result.data);
  } finally {
    removeTempDir(tempDir);
  }
});

test('configValidate - accepts custom config path', async (t) => {
  const tempDir = createTempDir();
  const customConfigDir = path.join(tempDir, 'custom');

  try {
    createPackageJson(tempDir);
    initGitRepo(tempDir);

    // Create config in custom location
    fs.mkdirSync(customConfigDir, { recursive: true });
    createConfigJson(customConfigDir);

    const customConfigPath = path.join(customConfigDir, 'repo.config.json');

    const result = await configValidate({
      root: tempDir,
      configPath: customConfigPath,
    });

    t.true(result.success, 'Should validate config from custom path');
    t.truthy(result.data);
  } finally {
    removeTempDir(tempDir);
  }
});
