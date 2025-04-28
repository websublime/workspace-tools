This rules are mandatory to apply to any answer given by AI.

- Assumptions, cannot be used. Always check apis and source code available. In case they are missing ask to the user to provide it.
- Problem resolution shouldn't be take simplistic. If we need to support all operating systems let's evaluate and create the source for them.
- Robust code, no simplistic approaches
- Consistency in code. Let's produce always the same patterns used between crates/packages.
- Documentation should be applied in module level, structs, properties and methods/functions. Provide always detail documentation for all and include examples on it.
- Clippy rules that are mandatory to use:

```rust
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
```

- Always prioritize clarity and maintainability over speed and brevity
- Remember to follow best practices for error handling and logging
- Reuse all the crates from the api specs if needed, and ensure that the code is well-documented and follows the Rust community's style guide
- Detail information, file location and so one