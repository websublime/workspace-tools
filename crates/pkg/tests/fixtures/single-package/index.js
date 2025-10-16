// test-single-package - Test single package fixture
// This is a simple test fixture for sublime_pkg_tools

const express = require('express');
const _ = require('lodash');
const axios = require('axios');

/**
 * Creates an Express application instance
 * @returns {Object} - Express app
 */
function createApp() {
  const app = express();

  app.get('/', (req, res) => {
    res.json({ message: 'Test single package', version: getVersion() });
  });

  app.get('/health', (req, res) => {
    res.json({ status: 'ok' });
  });

  return app;
}

/**
 * Processes data using lodash utilities
 * @param {Array} data - Input data
 * @returns {Object} - Processed data
 */
function processData(data) {
  return {
    unique: _.uniq(data),
    sorted: _.sortBy(data),
    grouped: _.groupBy(data, item => typeof item),
  };
}

/**
 * Fetches data from an API endpoint
 * @param {string} url - API endpoint URL
 * @returns {Promise<Object>} - Response data
 */
async function fetchData(url) {
  try {
    const response = await axios.get(url);
    return response.data;
  } catch (error) {
    throw new Error(`Failed to fetch data: ${error.message}`);
  }
}

/**
 * Gets the package version
 * @returns {string} - Package version
 */
function getVersion() {
  return '1.5.0';
}

/**
 * Gets package information
 * @returns {Object} - Package info
 */
function getPackageInfo() {
  return {
    name: 'test-single-package',
    version: getVersion(),
    description: 'Test single package fixture for sublime_pkg_tools',
    dependencies: {
      express: '^4.18.2',
      lodash: '^4.17.21',
      axios: '^1.4.0',
    },
  };
}

module.exports = {
  createApp,
  processData,
  fetchData,
  getVersion,
  getPackageInfo,
};
