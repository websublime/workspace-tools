//! Clone command implementation for Node.js bindings.
//!
//! # What
//!
//! This module implements the `clone` NAPI function that clones a Git repository
//! and optionally initializes it as a workspace. It handles SSH authentication
//! and workspace configuration in a single operation.
//!
//! # How
//!
//! The function:
//! 1. Validates the input parameters (root path, URL, destination)
//! 2. Calls `execute_clone` from `sublime_cli_tools`
//! 3. Captures the JSON output containing clone operation results
//! 4. Returns a `JsonResponse<CloneData>` with repository information
//!
//! # Why
//!
//! The clone command streamlines the process of setting up new workspace projects:
//! - Clones from any Git URL (HTTPS or SSH)
//! - Handles SSH key authentication
//! - Optionally initializes workspace configuration
//! - Supports shallow clones for faster operations
//!
//! # Examples
//!
//! ```typescript
//! import { clone } from '@websublime/workspace-tools';
//!
//! // Clone a public repository
//! const result = await clone({
//!   root: '.',
//!   url: 'https://github.com/org/repo.git',
//!   destination: 'my-project'
//! });
//!
//! if (result.success) {
//!   console.log(`Cloned to: ${result.data.path}`);
//!   console.log(`Branch: ${result.data.branch}`);
//! }
//!
//! // Clone with SSH and initialize workspace
//! const sshResult = await clone({
//!   root: '.',
//!   url: 'git@github.com:org/private-repo.git',
//!   destination: 'my-private-project',
//!   initialize: true,
//!   strategy: 'independent'
//! });
//!
//! if (sshResult.success) {
//!   console.log(`Cloned to: ${sshResult.data.path}`);
//!   if (sshResult.data.initialized) {
//!     console.log(`Config created at: ${sshResult.data.configPath}`);
//!   }
//! }
//!
//! // Shallow clone with custom SSH keys
//! const shallowResult = await clone({
//!   root: '.',
//!   url: 'git@github.com:org/repo.git',
//!   destination: 'shallow-clone',
//!   depth: 1,
//!   branch: 'main',
//!   sshKeyPaths: ['~/.ssh/id_ed25519', '~/.ssh/id_rsa']
//! });
//! ```

// TODO: will be implemented on story 9.3 - Clone Command
//
// Implementation outline:
//
// #[napi]
// pub async fn clone(params: CloneParams) -> JsonResponse<CloneData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Validate URL is not empty
//     if params.url.trim().is_empty() {
//         return JsonResponse::validation_error("url cannot be empty", Some("url"));
//     }
//
//     // 3. Create CloneArgs from params
//     let args = CloneArgs {
//         url: params.url.clone(),
//         destination: params.destination.clone(),
//         branch: params.branch.clone(),
//         depth: params.depth,
//         init: params.initialize.unwrap_or(false),
//         strategy: params.strategy.clone(),
//         ssh_key_paths: params.ssh_key_paths.clone(),
//         ..Default::default()
//     };
//
//     // 4. Create Output with JSON format for capturing
//     let mut buffer = Vec::new();
//     let output = Output::new(OutputFormat::Json, &mut buffer, false);
//
//     // 5. Determine base path
//     let root_path = Path::new(&params.root);
//     let config_path = None; // Will use default config discovery
//
//     // 6. Call execute_clone from CLI (commands/clone.rs)
//     match execute_clone(&args, &output, root_path, config_path).await {
//         Ok(()) => {
//             // 7. Parse JSON response from buffer
//             let json_str = String::from_utf8_lossy(&buffer);
//             match serde_json::from_str::<JsonResponse<CloneData>>(&json_str) {
//                 Ok(response) => response,
//                 Err(e) => JsonResponse::error(format!("Failed to parse clone response: {}", e)),
//             }
//         }
//         Err(e) => JsonResponse::error(format!("Clone failed: {}", e)),
//     }
// }
//
// Note: The clone command uses sublime_git_tools::Repo::clone_with_options
// for the underlying git operation. SSH authentication is handled by
// providing custom SSH key paths that are tried in order.
//
// When initialize is true, the init command is automatically called
// after a successful clone to set up the workspace configuration.
