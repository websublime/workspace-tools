# workspace-node-tools

[![Crates.io](https://img.shields.io/crates/v/workspace-node-tools.svg)](https://crates.io/crates/workspace-node-tools)
[![Docs.rs](https://docs.rs/workspace-node-tools/badge.svg)](https://docs.rs/workspace-node-tools)
[![CI](https://github.com/websublime/workspace-node-tools/workflows/CI/badge.svg)](https://github.com/websublime/workspace-node-tools/actions)

## About

This is a tool to help manage packages in a monorepo style. It can give info about packages existence, package manager defined (node), git helpers to check which package as changes, manage those changes thur a file (.changes.json), give output of conventional commit and changelog generation.

## Installation

`cargo install workspace-node-tools`

### Cargo

- Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
- run `cargo install workspace-node-tools`

### Claude Tools

- Memory Bank
  - claude mcp add --scope user memory-bank -e MEMORY_BANK_ROOT=~/.claude/memory-bank -- npx -y @allpepper/memory-bank-mcp
- Knowledge Graph
  - claude mcp add --scope user knowledge-graph -e GRAPH_STORAGE_PATH=~/.claude/knowledge-graph -- npx -y mcp-knowledge-graph
- Context7
  - claude mcp add --scope user context7 -- npx -y @upstash/context7-mcp

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Info

Template from [here](https://rust-github.github.io/)
