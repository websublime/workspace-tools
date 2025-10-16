// @test/pkg-a - Test package A
// This is a simple test fixture for sublime_pkg_tools

const _ = require('lodash');

/**
 * Example function for testing
 * @param {Array} items - Array of items to process
 * @returns {Array} - Processed items
 */
function processItems(items) {
  return _.uniq(items);
}

/**
 * Gets the package version
 * @returns {string} - Package version
 */
function getVersion() {
  return '1.0.0';
}

module.exports = {
  processItems,
  getVersion,
};
