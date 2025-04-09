//! # sublime_standard_tools
//!
//! A collection of utilities for working with Node.js projects, including command execution,
//! package manager detection, project root path discovery, and more.
//!
//! ## Overview
//!
//! `sublime_standard_tools` offers a robust set of tools for interacting with Node.js projects
//! from Rust applications. It enables seamless integration with Node.js ecosystems, allowing
//! Rust applications to:
//!
//! - Execute shell commands with proper error handling and output parsing
//! - Detect which package manager (npm, yarn, pnpm, or bun) is being used in a project
//! - Find the root of a project by locating package manager lock files
//! - Handle common string manipulations needed for command outputs
//!
//! ## Main Features
//!
//! ### Command Execution
//!
//! ```
//! use sublime_standard_tools::{execute, ComandResult};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Execute a git command and process its output
//! let version = execute("git", ".", ["--version"], |output, _| {
//!     Ok(output.to_string())
//! })?;
//!
//! println!("Git version: {}", version);
//! # Ok(())
//! # }
//! ```
//!
//! ### Package Manager Detection
//!
//! ```
//! use sublime_standard_tools::{detect_package_manager, CorePackageManager};
//! use std::path::Path;
//!
//! let project_dir = Path::new("./my-node-project");
//! match detect_package_manager(project_dir) {
//!     Some(CorePackageManager::Npm) => println!("Using npm"),
//!     Some(CorePackageManager::Yarn) => println!("Using yarn"),
//!     Some(CorePackageManager::Pnpm) => println!("Using pnpm"),
//!     Some(CorePackageManager::Bun) => println!("Using bun"),
//!     None => println!("No package manager detected"),
//! }
//! ```
//!
//! ### Project Root Discovery
//!
//! ```
//! use sublime_standard_tools::get_project_root_path;
//!
//! // Find the project root from the current directory
//! if let Some(root_path) = get_project_root_path(None) {
//!     println!("Project root: {}", root_path.display());
//! }
//! ```

mod command;
mod error;
mod manager;
mod path;
mod utils;

pub use command::{execute, ComandResult};
pub use error::CommandError;
pub use manager::{detect_package_manager, CorePackageManager, CorePackageManagerError};
pub use path::get_project_root_path;
pub use utils::strip_trailing_newline;
