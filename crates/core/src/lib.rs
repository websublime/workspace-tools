//! # workspace-core
//!
//! Core abstractions and detection mechanisms for JavaScript/TypeScript workspace management.
//!
//! ## What
//!
//! This crate provides the foundational abstractions and detection mechanisms for working
//! with JavaScript/TypeScript projects from Rust. It serves as the core building block
//! for the workspace-node-tools ecosystem.
//!
//! ## How
//!
//! The crate enables reliable detection of:
//! - **Repository Types**: Identifying the runtime ecosystem (Node, Deno, Bun) based on
//!   characteristic files
//! - **Package Managers**: Identifying which package manager (npm, yarn, pnpm, bun, deno)
//!   is used in a project
//! - **Repository Kinds**: Determining if a project is a single-package repository or a monorepo
//! - **Monorepo Analysis**: Detecting workspace configuration, discovering workspace packages,
//!   and analyzing internal dependencies
//!
//! ## Why
//!
//! Managing JavaScript/TypeScript projects requires understanding their structure and tooling.
//! This crate provides a unified, type-safe interface for detecting and analyzing project
//! configurations, enabling higher-level tools to work with any project structure consistently.
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_core::Project;
//!
//! // Detect and analyze a project
//! let project = Project::discover("/path/to/project")?;
//!
//! println!("Repository type: {:?}", project.repo_type());
//! println!("Package manager: {:?}", project.package_manager());
//! println!("Is monorepo: {}", project.is_monorepo());
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
