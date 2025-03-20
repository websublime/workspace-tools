// @ts-check
import { getProjectRootPath } from '../dist/esm/index.mjs';

/**
 * Use the getProjectRootPath function
 * @type {import('../dist/types').getProjectRootPath}
 */
const rootPath = getProjectRootPath();

console.log(rootPath);
