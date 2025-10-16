// @test/pkg-b - Test package B
// This is a simple test fixture for sublime_pkg_tools

const React = require('react');
const pkgA = require('@test/pkg-a');

/**
 * Example React component wrapper for testing
 * @param {Object} props - Component props
 * @returns {Object} - React element
 */
function createComponent(props) {
  return React.createElement('div', props, 'Test Component');
}

/**
 * Uses functionality from pkg-a
 * @param {Array} items - Array of items
 * @returns {Array} - Processed items
 */
function processWithPkgA(items) {
  return pkgA.processItems(items);
}

/**
 * Gets the package version
 * @returns {string} - Package version
 */
function getVersion() {
  return '2.0.0';
}

/**
 * Gets dependency versions
 * @returns {Object} - Dependency versions
 */
function getDependencyVersions() {
  return {
    pkgA: pkgA.getVersion(),
    react: React.version,
  };
}

module.exports = {
  createComponent,
  processWithPkgA,
  getVersion,
  getDependencyVersions,
};
