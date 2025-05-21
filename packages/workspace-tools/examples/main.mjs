// @ts-check
import { getVersion, MonorepoProject } from '../dist/esm/index.mjs';

/**
 * @typedef {import('../dist/types').getVersion} getVersion
 * @typedef {import('../dist/types').MonorepoProject} MonorepoProject
 */

/** @type {getVersion} */
const version = getVersion();

console.log(version);

try {
  /** @type {MonorepoProject} */
  const monorepoProject = new MonorepoProject();
  const description = monorepoProject.getProjectDescription();
  const validation = monorepoProject.validate();
  const workspaceDescriptor = monorepoProject.getWorkspaceDescriptor();
  console.log(description);
  console.log(validation);
  console.log(workspaceDescriptor);
  console.log(monorepoProject.getPackageDescriptor('@websublime/workspace-tools'));
  console.log(monorepoProject.getWorkspaceDependencyGraph());
} catch (e) {
  console.error('Error:', e);
}
