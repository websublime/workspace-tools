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
//! The crate is organized into modules that each focus on a specific concern:
//! - `command`: Execution and management of shell commands
//! - `error`: Comprehensive error types and handling utilities
//! - `project`: Project structure detection and navigation
//! - (other modules to be implemented)
//!
//! ## Why
//! Interacting with Node.js projects from Rust typically involves a significant
//! amount of boilerplate code for path handling, command execution, and project
//! structure detection. This crate abstracts these common tasks into a reusable
//! library with consistent error handling and cross-platform support.

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

/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of the crate
#[must_use]
pub fn version() -> &'static str {
    VERSION
}
