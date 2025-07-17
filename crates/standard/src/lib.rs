//! # `sublime_standard_tools`
//!
//! A comprehensive toolkit for working with Node.js projects from Rust applications.
//!
//! ## What
//! This crate provides a foundational set of utilities for interacting with Node.js
//! projects, package managers, and development workflows from Rust. It handles
//! project structure detection, command execution, environment management,
//! and various other tasks required when working with Node.js ecosystems.
//!
//! ## How
//! The crate follows a clean architectural approach with clear separation of concerns:
//!
//! ### Core Modules
//! - **`node`**: Generic Node.js concepts (repositories, package managers)
//! - **`project`**: Unified project detection and management
//! - **`monorepo`**: Monorepo-specific functionality and workspace management
//! - **`command`**: Robust command execution framework
//! - **`filesystem`**: Safe filesystem operations and path utilities
//! - **`error`**: Comprehensive error handling
//!
//! ### Architecture Overview
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    sublime_standard_tools                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │  project/     │  Unified project detection and management   │
//! │  ├─detector   │  ├─ ProjectDetector (any project type)     │
//! │  ├─manager    │  ├─ ProjectManager (lifecycle management)  │
//! │  └─types      │  └─ ProjectInfo trait (common interface)   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  node/        │  Generic Node.js concepts                  │
//! │  ├─types      │  ├─ RepoKind (Simple vs Monorepo)         │
//! │  ├─package_*  │  ├─ PackageManager & PackageManagerKind   │
//! │  └─repository │  └─ RepositoryInfo trait                  │
//! ├─────────────────────────────────────────────────────────────┤
//! │  monorepo/    │  Monorepo-specific functionality           │
//! │  ├─detector   │  ├─ MonorepoDetector (workspace detection) │
//! │  ├─descriptor │  ├─ MonorepoDescriptor (full structure)    │
//! │  └─kinds      │  └─ MonorepoKind (npm, yarn, pnpm, etc.)  │
//! ├─────────────────────────────────────────────────────────────┤
//! │  command/     │  Robust command execution                  │
//! │  filesystem/  │  Safe filesystem operations               │
//! │  error/       │  Comprehensive error handling             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Why
//! Interacting with Node.js projects from Rust typically involves a significant
//! amount of boilerplate code for path handling, command execution, and project
//! structure detection. This crate abstracts these common tasks into a reusable
//! library with consistent error handling and cross-platform support.
//!
//! The new architecture provides:
//! - **Clean separation of concerns**: Each module has a clear responsibility
//! - **Unified project handling**: Works with both simple and monorepo projects
//! - **Type-safe abstractions**: Strong typing prevents common errors
//! - **Extensible design**: Easy to add new project types and package managers
//!
//! ## Quick Start
//!
//! ### Detect any Node.js project
//! ```rust
//! use sublime_standard_tools::project::{ProjectDetector, ProjectConfig};
//! use std::path::Path;
//!
//! let detector = ProjectDetector::new();
//! let config = ProjectConfig::new();
//!
//! match detector.detect(Path::new("."), &config) {
//!     Ok(project) => {
//!         let info = project.as_project_info();
//!         println!("Found {} project", info.kind().name());
//!         
//!         if let Some(pm) = info.package_manager() {
//!             println!("Using {} package manager", pm.command());
//!         }
//!     }
//!     Err(e) => eprintln!("Detection failed: {}", e),
//! }
//! ```
//!
//! ### Work with package managers
//! ```rust
//! use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
//! use std::path::Path;
//!
//! // Detect package manager
//! let manager = PackageManager::detect(Path::new("."))?;
//! println!("Using {}", manager.command());
//!
//! // Check capabilities
//! if manager.supports_workspaces() {
//!     println!("Workspaces supported");
//! }
//! ```
//!
//! ### Analyze monorepos
//! ```rust
//! use sublime_standard_tools::monorepo::MonorepoDetector;
//! use std::path::Path;
//!
//! let detector = MonorepoDetector::new();
//! if let Some(kind) = detector.is_monorepo_root(".")? {
//!     let monorepo = detector.detect_monorepo(".")?;
//!     
//!     println!("Found {} with {} packages", 
//!              monorepo.kind().name(),
//!              monorepo.packages().len());
//!              
//!     // Analyze dependencies
//!     let graph = monorepo.get_dependency_graph();
//!     for (pkg, deps) in graph {
//!         println!("{} has {} dependents", pkg, deps.len());
//!     }
//! }
//! ```

#![doc = include_str!("../SPEC.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

pub mod command;
pub mod error;
pub mod filesystem;
pub mod monorepo;
pub mod node;
pub mod project;

/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of the crate
#[must_use]
pub fn version() -> &'static str {
    VERSION
}
