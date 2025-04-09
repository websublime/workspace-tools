//! Workspace management for monorepos.
//!
//! This module provides functionality for working with monorepo workspaces,
//! including discovering packages, analyzing dependencies, and managing
//! workspace-wide operations.
//!
//! ## Key Components
//!
//! - **Workspace**: Central struct for managing packages in a monorepo
//! - **WorkspaceManager**: High-level interface for creating and analyzing workspaces
//! - **WorkspaceConfig**: Configuration for workspace settings
//! - **DiscoveryOptions**: Options for package discovery
//! - **ValidationOptions**: Options for workspace validation
//! - **WorkspaceGraph**: Analysis results for workspace dependencies
//! - **WorkspaceAnalysis**: Analysis of workspace health and structure
//!
//! ## Example Usage
//!
//! ```no_run
//! use sublime_monorepo_tools::{WorkspaceManager, DiscoveryOptions};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a workspace manager
//! let manager = WorkspaceManager::new();
//!
//! // Discover a workspace with custom options
//! let options = DiscoveryOptions::new()
//!     .include_patterns(vec!["packages/*/package.json"])
//!     .exclude_patterns(vec!["**/tests/**"]);
//!
//! let workspace = manager.discover_workspace(".", &options)?;
//!
//! // Analyze the workspace
//! let analysis = manager.analyze_workspace(&workspace)?;
//!
//! // Check for dependency cycles
//! let cycles = workspace.get_circular_dependencies();
//! if !cycles.is_empty() {
//!     println!("Found {} dependency cycles", cycles.len());
//!     for cycle in cycles {
//!         println!("Cycle: {}", cycle.join(" → "));
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod analysis;
pub mod config;
pub mod discovery;
pub mod error;
pub mod graph;
pub mod manager;
pub mod validation;
pub mod workspace;
