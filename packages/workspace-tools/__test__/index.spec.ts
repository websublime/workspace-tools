/**
 * Integration tests for the index module exports.
 *
 * ## What
 *
 * This module contains basic integration tests that verify the main exports
 * from the workspace-tools package are working correctly.
 *
 * ## How
 *
 * Tests verify that:
 * - The `getVersion()` function returns a valid semver version string
 * - The `status` and `init` functions are exported and callable
 *
 * ## Why
 *
 * These smoke tests ensure that the native bindings are loaded correctly
 * and the basic exports are functional. They serve as a first line of
 * defense against packaging or binding issues.
 *
 * @packageDocumentation
 */

import test from 'ava';

import { getVersion, status, init } from '../src/index';

test('getVersion - returns a valid version string', (t) => {
  const version = getVersion();

  t.is(typeof version, 'string', 'Version should be a string');
  t.true(version.length > 0, 'Version should not be empty');

  // Version should be a valid semver (basic check)
  const semverPattern = /^\d+\.\d+\.\d+(-[\w.]+)?(\+[\w.]+)?$/;
  t.regex(version, semverPattern, `Version "${version}" should be a valid semver`);
});

test('status function is exported and callable', (t) => {
  t.is(typeof status, 'function', 'status should be a function');
});

test('init function is exported and callable', (t) => {
  t.is(typeof init, 'function', 'init should be a function');
});
