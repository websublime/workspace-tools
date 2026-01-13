# workspace-core

Core abstractions and detection mechanisms for JavaScript/TypeScript workspace management.

## Overview

This crate provides the foundational abstractions and detection mechanisms for working with JavaScript/TypeScript projects from Rust. It serves as the core building block for the workspace-node-tools ecosystem.

## Features

- **Repository Type Detection**: Identify the runtime ecosystem (Node, Deno, Bun) based on characteristic files
- **Package Manager Detection**: Identify which package manager (npm, yarn, pnpm, bun, deno) is used in a project
- **Repository Kind Detection**: Determine if a project is a single-package repository or a monorepo
- **Monorepo Analysis**: Detect workspace configuration, discover workspace packages, and analyze internal dependencies
- **Core Abstractions**: Reusable types and traits for the workspace-node-tools ecosystem

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
workspace-core = "0.1"
```

## Usage

```rust
use workspace_core::Project;

// Detect and analyze a project
let project = Project::discover("/path/to/project")?;

println!("Repository type: {:?}", project.repo_type());
println!("Package manager: {:?}", project.package_manager());
println!("Is monorepo: {}", project.is_monorepo());

// Access workspace packages in a monorepo
for package in project.workspace_packages() {
    println!("Package: {} @ {}", package.name(), package.version());
}
```

## Dependencies

This crate depends on:

- [`workspace-fs`](../filesystem) - Filesystem abstraction layer

## License

MIT