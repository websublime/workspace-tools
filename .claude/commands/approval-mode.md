# Claude Code Approval Mode

**ACTIVATE APPROVAL MODE**: From now on, you work in "Approval Mode". This means:

## Rules for Major Steps

Before executing any of the following **major steps**, you must:

0. **Plan** use sequential-thinking to plan the next step
1. **Announce** what you plan to do
2. **Explain** why this step is necessary
3. **Show** exactly what will happen
4. **Wait** for my explicit confirmation
5. **Execute** use ultrathink mode to implement the step if approved

### Major Steps Include

- Creating new files or folders
- Deleting or renaming existing files
- Installing or removing packages/dependencies
- Major code refactoring (>20 lines)
- Git operations (commit, push, pull, etc.)
- Logic and functional decisions
- implementing a feature chunk

### Format for Approval Requests

```
🔄 APPROVAL REQUIRED
What: [Brief description of the action]

Why: [Justification/necessity]

Details: [Exact steps or code to be executed]

Impact: [What will change as a result]

Should I proceed? (yes/no)
```

## Minor Steps (NO Approval Needed)

- Code formatting and styling
- Small bug fixes (<10 lines)
- Adding/changing comments
- Renaming variables
- Adjusting import statements
- run linting, typechecking and tests

## Behavior

- **STOP** after every approval request and wait for response
- **NEVER** execute major steps without confirmation
- On "no" - explain alternatives
- On "yes" - execute the step and confirm completion
- when uou get the approval and run the implementation, always run linting and typechecking

## Session Compact/Summary

When creating a compact or summary for a new session, **ALWAYS include this approval mode configuration** in the summary so that the new session continues with the same controlled workflow.

**Please confirm that you understand and have activated this mode.**