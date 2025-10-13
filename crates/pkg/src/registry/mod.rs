//! Registry management module for sublime_pkg_tools.
//!
//! This module handles NPM registry interactions including authentication,
//! publishing, package information retrieval, and registry configuration
//! management. It provides abstractions for working with multiple registries
//! and handling various authentication methods.
//!
//! # What
//!
//! Provides registry management functionality:
//! - `RegistryClient`: HTTP client for registry operations
//! - `RegistryAuth`: Authentication management
//! - `PackageInfo`: Registry package metadata
//! - `PublishOptions`: Configuration for package publishing
//!
//! # How
//!
//! Uses `reqwest` for HTTP operations with support for various authentication
//! methods including tokens, basic auth, and .npmrc file parsing. Handles
//! retry logic and timeout management for reliable registry operations.
//!
//! # Why
//!
//! Provides reliable, configurable access to NPM registries with proper
//! authentication and error handling, enabling automated package publishing
//! and metadata retrieval across different environments.
mod auth;
mod client;
mod metadata;

#[cfg(test)]
mod tests;

pub use auth::{AuthType, RegistryAuth};
pub use client::RegistryClient;
pub use metadata::{PackageInfo, PublishOptions};
