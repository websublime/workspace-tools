# Architecture Decision Records

This document records significant architectural decisions for the workspace-node-tools project. Each decision is numbered and follows a consistent structure.

---

## ADR-001: Native async traits without dyn dispatch

**Status:** Accepted  
**Date:** 2026-02-07  
**Scope:** `workspace-fs` crate (`FileSystem` trait)

### Context

The `FileSystem` trait in `workspace-fs` is the foundational abstraction for all filesystem operations across the workspace-node-tools ecosystem. The original design (per PRD revision 0) specified `#[async_trait]` from the `async-trait` crate to enable async methods in traits.

However, the project targets:
- **Rust Edition 2024** with `rust-version = "1.90.0"` MSRV
- Native `async fn` in traits was stabilized in Rust 1.75
- Return-type notation and `Send` bound inference improved through 1.85+
- Edition 2024 captures all lifetimes in `impl Trait` return types, further simplifying async trait usage

The `async-trait` crate works by desugaring `async fn` into `-> Pin<Box<dyn Future + Send>>`, which imposes:
- A heap allocation per method call
- A proc-macro compile-time dependency
- Inability to benefit from future compiler optimizations to native async traits

### Decision

Use **native `async fn` in traits** for the `FileSystem` trait. Require `Send + Sync` as supertraits. Consumers use **generic bounds** (`<FS: FileSystem>` or `impl FileSystem`) rather than trait objects (`dyn FileSystem`).

```rust
// The trait definition
pub trait FileSystem: Send + Sync {
    async fn read_to_string(&self, path: &Path) -> Result<String>;
    // ... 23 more methods
}

// Consumer pattern: generics, not dyn
async fn read_config<FS: FileSystem>(fs: &FS, path: &Path) -> Result<String> {
    fs.read_to_string(path).await
}
```

### Rationale

1. **Zero overhead**: No `Box::pin()` allocation per call. The compiler monomorphizes each generic instantiation.
2. **No proc-macro dependency**: Removes `async-trait` from the dependency tree, reducing compile times and supply-chain surface.
3. **Forward-compatible**: Aligns with the Rust project's direction. As async trait support matures (e.g., `dyn`-compatible async traits, trait-variant), this codebase benefits automatically.
4. **Simpler error messages**: Native async traits produce clearer compiler diagnostics than proc-macro-generated code.

### Consequences

**Positive:**
- Faster method dispatch (no heap allocation, no vtable)
- Smaller dependency tree (no `async-trait`, `syn`, `quote` transitive deps)
- Cleaner generated documentation (methods show as `async fn`, not `fn -> Pin<Box<...>>`)

**Negative:**
- Cannot use `dyn FileSystem` directly. This means:
  - No heterogeneous collections of `Box<dyn FileSystem>`
  - No runtime-polymorphic dispatch without a manual wrapper
- If `dyn` dispatch is needed later, options include:
  1. The [`trait-variant`](https://crates.io/crates/trait-variant) crate to auto-generate a dyn-compatible variant
  2. A manual `DynFileSystem` wrapper struct that boxes futures internally
  3. An enum dispatch pattern (`enum AnyFs { Real(RealFileSystem), Mock(MockFileSystem) }`)

**Mitigations:**
- The ecosystem currently has exactly two implementations (`RealFileSystem`, `MockFileSystem`), so `dyn` dispatch is not needed
- All consumer code uses generics, which is idiomatic Rust and the recommended pattern for async traits
- If a third-party plugin system is added later, enum dispatch or `trait-variant` can be introduced without changing the core trait

### References

- [Rust Blog: Async fn in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html)
- [Edition 2024 migration guide](https://doc.rust-lang.org/edition-guide/rust-2024/)
- [trait-variant crate](https://crates.io/crates/trait-variant)
- PRD Revision Appendix A (`crates/filesystem/PRD.md`): "Native async trait" revision
