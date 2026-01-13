# workspace-fs

A unified filesystem abstraction layer for the workspace-node-tools ecosystem.

## Overview

This crate provides a trait-based abstraction over filesystem operations, enabling consistent filesystem access, comprehensive testing through mock implementations, and high-performance asynchronous I/O for large monorepo operations.

## Features

- **Async-First Design**: Built on `tokio::fs` for high-performance async I/O
- **Trait-Based Abstraction**: `FileSystem` trait enables dependency injection and mocking
- **Mock Implementation**: In-memory filesystem for fast, deterministic unit tests
- **Cross-Platform**: Abstracts platform-specific filesystem quirks
- **Error Handling**: Rich error types with context using `snafu`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
workspace-fs = "0.1"
```

## Usage

### Production Code

```rust
use workspace_fs::{FileSystem, RealFileSystem};

async fn read_config(fs: &impl FileSystem) -> Result<String, workspace_fs::Error> {
    fs.read_to_string("config.json").await
}

#[tokio::main]
async fn main() {
    let fs = RealFileSystem::new();
    match read_config(&fs).await {
        Ok(content) => println!("Config: {}", content),
        Err(e) => eprintln!("Failed to read config: {}", e),
    }
}
```

### Testing with Mock Filesystem

```rust
use workspace_fs::{FileSystem, MockFileSystem};

#[tokio::test]
async fn test_read_config() {
    let mut fs = MockFileSystem::new();
    fs.add_file("config.json", r#"{"key": "value"}"#);

    let content = fs.read_to_string("config.json").await.unwrap();
    assert_eq!(content, r#"{"key": "value"}"#);
}
```

## API Overview

### Core Trait

- `FileSystem` - Async trait defining all filesystem operations

### Implementations

- `RealFileSystem` - Production implementation using `tokio::fs`
- `MockFileSystem` - In-memory implementation for testing

### Operations

| Category | Operations |
|----------|------------|
| **Read** | `read_to_string`, `read` |
| **Write** | `write`, `create_dir`, `create_dir_all` |
| **Metadata** | `exists`, `is_file`, `is_dir`, `metadata` |
| **Directory** | `read_dir`, `walk_dir` |
| **Path** | `canonicalize`, `normalize` |

## License

MIT License - see [LICENSE](../../LICENSE.md) for details.