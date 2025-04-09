//! Task management system for monorepos.
//!
//! This module provides functionality for defining, organizing, and executing tasks
//! within a monorepo workspace. It supports dependency resolution, parallel execution,
//! and flexible configuration options.
//!
//! # Key Components
//!
//! - **Task**: Represents a command to be executed with its dependencies and configuration
//! - **TaskRunner**: Manages task execution within a workspace
//! - **TaskGraph**: Handles dependency relationships between tasks
//! - **TaskFilter**: Filters tasks based on names, patterns, and package associations
//!
//! # Examples
//!
//! ```no_run
//! use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
//!
//! # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
//! // Create a task runner
//! let mut runner = TaskRunner::new(workspace);
//!
//! // Define tasks with dependencies
//! runner.add_tasks(vec![
//!     Task::new("build", "npm run build"),
//!     Task::new("test", "npm test").with_dependency("build"),
//!     Task::new("lint", "npm run lint"),
//!     Task::new("deploy", "npm run deploy").with_dependencies(vec!["build", "test"])
//! ]);
//!
//! // Run a specific task (and its dependencies)
//! runner.run_task("deploy")?;
//!
//! // Create a graph visualization
//! let graph = runner.build_task_graph()?;
//! let levels = graph.task_levels();
//! println!("Task graph has {} levels", levels.len());
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod filter;
pub mod graph;
pub mod info;
pub mod parallel;
pub mod runner;
pub mod task;
