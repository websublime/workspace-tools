## Collaboration Guidelines
- **Challenge and question**: Don't immediately agree or proceed with requests that seem suboptimal, unclear, or potentially problematic
- **Push back constructively**: If a proposed approach has issues, suggest better alternatives with clear reasoning
- **Think critically**: Consider edge cases, performance implications, maintainability, and best practices before implementing
- **Seek clarification**: Ask follow-up questions when requirements are ambiguous or could be interpreted multiple ways
- **Propose improvements**: Suggest better patterns, more robust solutions, or cleaner implementations when appropriate
- **Be a thoughtful collaborator**: Act as a good teammate who helps improve the overall quality and direction of the project

# Rust Rules

This rules are mandatory to apply to any answer given by AI.

- Language is english
- Assumptions (MANDATORY), cannot be used. Always check apis and source code available. In case they are missing ask to the user to provide it.
- Problem resolution shouldn't be take simplistic. If we need to support all operating systems let's evaluate and create the solution for them.
- Robust code, no simplistic approaches, no placeholders, no "in a real case we would do this or that", no "this is just an example", no "this is a placeholder", no "this is not implemented yet". Always provide a complete solution and the goal is enterprise level.
- Consistency in code. Let's produce always the same patterns used between crates/packages.
- Documentation should be in English, applied in module level, structs, properties and methods/functions. Provide always detail documentation for all and include examples on it. Code blocks in files should describe initial the overall of the file and anwser these three topics: What, How and why.
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
- Detail information, file location and no methods with no implementation or saying in a real case we would use this or that or even doing this or that.
- When clippy rules clash with implementation, always prefer to follow clippy rules, if you can't let' signed with a comment explaining why the rule was not followed and allow the exception.
- Whenever there's a doubt about what decision to make, instead of making the decision, ask the user for clarification. This ensures that the solution aligns with their expectations and requirements.
- Internal modules must use pub(crate) visibility, so they are only accessible within the crate, not outside of it.

# Additional instructions

- base crates
  - sublime-standard-tools:
    - directory: crates/standard
    - spec: crates/standard/SPEC.md
  - sublime-package-tools:
    - directory: crates/pkg
    - spec: crates/pkg/SPEC.md
  - sublime-git-tools:
    - directory: crates/git
    - spec: crates/git/SPEC.md
  - sublime-workspace-tools: (To be done)
    - directory: crates/workspace
    - spec: crates/workspace/SPEC.md
    
- Clippy always 100% and tests 100% coverage