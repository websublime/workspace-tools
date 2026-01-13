# Session Workflow Template

This document defines the systematic workflow for each development session. Follow these instructions precisely to maintain consistency, quality, and traceability across all development work.

## Prompt for you (AI)

As a senior developer you will pick up the next open task from beads and initiate or continue the work. You will first follow the guidelines in this document as rules of gold and in any case of doubt ask the user.
Keep the work close to the scope of the task. If you  have dependency of any future implementation add the implementation and on the context use todo! refering in the comment as "TODO: will be implemented on story/task/epic N", also skip the todo for clippy reason on that context only. Analyse very well the solution in detail and keep solutions very robust. Do not make assumptions always consult documentation and maintain the same code style, patterns and reuse code (DRY PRINCIPLE). Remember one task -> one branch. Go deeply and consistent.

---

## 🎯 Session Objective

Each session focuses on **one task** and its subtasks. Work is atomic, reviewed, and committed per subtask.

---

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
- Documentation should be in English, applied in module level, structs, properties and methods/functions. Provide always detail documentation for all and include examples on it. Code blocks in files should describe initial the overall of the file and answer these three topics: What, How and why.
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

## 📋 Pre-Session Checklist

Before starting any work, verify:

- [ ] You are in the project root directory
- [ ] Git is clean (`git status` shows no uncommitted changes)
- [ ] bd is synchronized (`bd sync`)

---

## 🚀 Phase 1: Session Initialization

### 1.1 Discover Current State

```bash
# Check for work in progress
bd list --status in_progress --json

# Check current branch
git branch --show-current

# Check git status
git status
```

### 1.2 Decision Tree

**If there's a task in_progress:**
- Continue from where it left off
- Identify the next open subtask

**If starting fresh (no in_progress tasks):**
- Checkout and pull main
- Identify the next ready task from Phase 0 (or current phase)

```bash
# Switch to main and update
git checkout feat/next-evolution
git pull origin feat/next-evolution
bd sync

# Find next ready task
bd ready --json
```

### 1.3 Setup Task Branch

```bash
# Create feature branch (naming convention: feature/p0-XX-short-description)
git checkout -b feature/<task-id-kebab-case>
```

### 1.4 Start Task

```bash
# Set task to in_progress
bd update <task-id> --status in_progress --json

# Sync bd
bd sync
```

### 1.5 Load Task Context

**CRITICAL**: Always read the PLAN.md section for the task before implementation. 
**MANDATORY**: If the document is not found ask the user where can i get context for the task.

```bash
# Get task details to find PLAN.md line references
bd show <task-id> --json
```

Then read the relevant section from `history/PLAN.md or docs/PLAN.md` using the line numbers from the task description (e.g., `📖 history/PLAN.md#L533-950`).
The plan file can be relative to the crate folder. Beads task or parent epic should refer the crate that belongs the task. If not founded please stop and ask the user about the file.

---

## 🔄 Phase 2: Subtask Loop

Repeat this loop for each subtask until all are complete.

### 2.1 Identify Next Subtask

```bash
# Show task with dependents (subtasks)
bd show <parent-task-id> --json
```

Pick the first subtask with `status: "open"`.

### 2.2 Start Subtask

```bash
# Set subtask to in_progress
bd update <subtask-id> --status in_progress --json
```

### 2.3 Implement

Execute the subtask according to its description and the PLAN.md specifications.

**During implementation, ensure:**
- Follow existing code patterns and conventions
- Add proper documentation (module-level, functions, types)
- Handle errors appropriately
- Consider edge cases
- **UNIT TESTS**: Tests are not implemented in implementations files, each module as a tests.rs file and tests are grouped inside.
- **E2E TESTS**: E2E tests are placed in crate directory tests, name convention goes <feature>_e2e.rs.

### 2.4 Review (Critical Step)

Before committing, perform a thorough review:

#### 2.4.1 Code Quality Review

- [ ] **Robustness**: Does the code handle errors and edge cases properly?
- [ ] **Improvements**: Can readability, performance, or patterns be improved?
- [ ] **Consistency**: Does it follow the project's existing patterns?
- [ ] **Documentation**: Are modules, structs, functions, and types documented?
- [ ] **Clean Code**: No dead code, TODOs, or temporary comments?
- [ ] **No Assumptions**: All APIs and sources verified, not assumed?

#### 2.4.2 Acceptance Criteria Check

Review the subtask description and verify all requirements are met.

#### 2.4.3 Technical DoD (Definition of Done)

Run validations always:

- Lint, clippy rules
- Format with fmt
- Test are covered (unit and e2e)

### 2.5 Decision Point

**If review PASSES:**
- Proceed to commit

**If review FAILS:**
- Fix the issues
- Return to step 2.4

### 2.6 Atomic Commit

```bash
# Stage changes
git add <files>

# Commit with conventional commits format
git commit -m "<type>(<scope>): <description>

<body - what was done>

Closes: <subtask-id>"
```

**Conventional Commits Reference:**
- `feat`: New feature
- `fix`: Bug fix
- `chore`: Maintenance tasks
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Adding tests

**Scope**: Use subtask ID (e.g., `P0-03.2`)

### 2.7 Close Subtask

```bash
# Close with descriptive reason
bd close <subtask-id> --reason "<what was accomplished>" --json

# Sync bd
bd sync
```

### 2.8 Loop Back

Return to **2.1** to process the next subtask.

---

## ✅ Phase 3: Task Completion

When all subtasks are closed:

### 3.1 Verify All Subtasks Complete

```bash
# Show task - all dependents should be closed
bd show <task-id> --json
```

### 3.2 Close Parent Task

```bash
bd close <task-id> --reason "<summary of all work completed>" --json
bd sync
```

### 3.3 Push Branch

```bash
git push -u origin <branch-name>
```

---

## 🏁 Phase 4: Session End

### 4.1 Final Sync

```bash
bd sync
```

### 4.2 Generate Session Summary

Provide a summary including:
- Task completed
- All subtasks completed with brief descriptions
- Files created/modified
- Commits made
- Branch name for PR

### 4.3 Next Steps

Indicate:
- Branch ready for PR/merge
- Next task to pick up in the next session

---

## 📚 Quick Reference

### bd Commands

```bash
bd ready --json                    # Find unblocked tasks
bd show <id> --json                # Show task details with subtasks
bd update <id> --status in_progress --json  # Start work
bd close <id> --reason "..." --json         # Complete work
bd sync                            # Sync with git
bd list --status in_progress --json         # Find WIP
```

### Git Commands

```bash
git checkout -b feature/<name>     # Create feature branch
git add <files>                    # Stage changes
git commit -m "..."                # Commit
git push -u origin <branch>        # Push branch
git checkout main                  # Switch to main
git pull origin main               # Update main
```

### Conventional Commit Template

```
<type>(<scope>): <short description>

<detailed description of what was done>

Closes: <issue-id>
```

---

## ⚠️ Important Rules

1. **Never assume** - Always check APIs, source code, and documentation
2. **One subtask at a time** - Complete and commit before moving to next
3. **Always review** - Code quality check before every commit
4. **Atomic commits** - Each commit corresponds to exactly one subtask
5. **Descriptive close reasons** - Document what was accomplished
6. **Sync bd frequently** - After every subtask completion
7. **Read the PLAN** - Always load context from PLAN.md before implementation
8. **Create changeset early** - Run `workspace changeset create` right after creating the feature branch

---

## 🔧 Troubleshooting

### bd sync fails
```bash
bd doctor --fix
bd sync
```

### Git conflicts
```bash
git stash
git pull origin feat/next-evolution
git stash pop
# Resolve conflicts manually
```

### Need to abandon subtask work
```bash
git checkout -- .                  # Discard all changes
bd update <subtask-id> --status open --json  # Reset status
```
