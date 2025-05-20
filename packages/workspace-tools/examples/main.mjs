// @ts-check
import { getVersion } from '../dist/esm/index.mjs';

/**
 * @typedef {import('../dist/types').getVersion} getVersion
 */

/** @type {getVersion} */
const version = getVersion();

console.log(version);
