//! # Project Management Module
//!
//! ## What
//! This module provides a unified API for detecting, managing, and working with
//! Node.js projects, regardless of whether they are simple repositories or monorepos.
//! It abstracts common functionality and provides a consistent interface for all
//! project types, including configuration management across different scopes.
//!
//! ## How
//! The module defines traits and types that represent project structures and provides
//! implementations for different project types. It uses a detector pattern to identify
//! project types and returns appropriate implementations through a common interface.
//! Configuration management is handled through a flexible system supporting multiple
//! scopes and file formats.
//!
//! ## Why
//! Node.js projects share many common characteristics regardless of their structure.
//! This module eliminates code duplication and provides a consistent API for working
//! with any Node.js project, making it easier to build tools that work across
//! different project types. Configuration management is also a cross-cutting concern
//! that applies to all project types.

mod configuration;
mod detector;
mod manager;
mod simple;
mod types;
mod validator;

#[cfg(test)]
mod tests;

pub use detector::ProjectDetector;
pub use manager::ProjectManager;
pub use simple::SimpleProject;
pub use types::{
    ConfigFormat, ConfigManager, ConfigScope, ConfigValue, GenericProject, ProjectConfig,
    ProjectDescriptor, ProjectInfo, ProjectKind, ProjectValidationStatus,
};
pub use validator::ProjectValidator;