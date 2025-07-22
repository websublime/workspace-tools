//! # Node.js Path Types
//!
//! ## What
//! This module defines path types specific to Node.js projects, providing
//! a type-safe way to reference common project directories and files.
//!
//! ## How
//! The `NodePathKind` enum represents different types of paths commonly
//! found in Node.js projects with descriptive variants.
//!
//! ## Why
//! Type-safe path handling prevents errors and makes code more readable
//! when dealing with Node.js project structures.

/// Represents common directory and file types in Node.js projects.
///
/// This enum provides a type-safe way to reference conventional Node.js
/// project paths like `"node_modules"`, `"src"`, etc.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::filesystem::NodePathKind;
///
/// let project_dir = Path::new("/project");
/// let node_modules = project_dir.join(NodePathKind::NodeModules.default_path());
/// assert_eq!(node_modules, Path::new("/project/node_modules"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodePathKind {
    /// Node modules directory
    NodeModules,
    /// Package configuration
    PackageJson,
    /// Source directory
    Src,
    /// Distribution directory
    Dist,
    /// Test directory
    Test,
}
