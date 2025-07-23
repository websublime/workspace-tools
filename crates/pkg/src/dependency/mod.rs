//! # Dependency Module
//!
//! This module provides functionality for working with package dependencies in Node.js projects.
//!
//! ## Overview
//!
//! The dependency module includes several components:
//!
//! - **Core Dependency**: Representation of a package dependency with name and version requirements
//! - **Dependency Change**: Tracking changes between dependency versions
//! - **Dependency Filtering**: Filtering dependencies by type (production, development, etc.)
//! - **Dependency Graph**: Graph-based representation of package dependencies
//! - **Dependency Registry**: Managing collections of dependencies
//! - **Dependency Resolution**: Resolving version conflicts between dependencies
//! - **Dependency Storage**: Thread-safe storage service for dependency data
//! - **Dependency Updates**: Applying version updates to dependencies
//!
//! These components work together to provide a comprehensive system for managing
//! package dependencies in JavaScript/TypeScript projects.

pub mod change;
pub mod conflict_resolver;
pub mod dependency;
pub mod filter;
pub mod graph;
pub mod registry;
pub mod registry_client;
pub mod resolution;
pub mod storage;
pub mod update;
