# CLI Documentation Scripts

This directory contains scripts for generating and maintaining CLI documentation.

## Scripts

### `generate-command-docs.sh`

Generates command reference documentation from CLI help text.

**Purpose:**
- Extracts help text from the compiled CLI binary
- Formats output into markdown documentation
- Ensures documentation stays in sync with implementation

**Usage:**

```bash
# 1. Build the CLI first
cargo build --release

# 2. Run the script
./crates/cli/scripts/generate-command-docs.sh
```

**Output:**
- `crates/cli/docs/COMMANDS_GENERATED.md` - Auto-generated from help text

**Comparison:**

The script generates `COMMANDS_GENERATED.md` which contains the raw CLI help text.
Compare this with the manually maintained `COMMANDS.md`:

```bash
diff crates/cli/docs/COMMANDS.md crates/cli/docs/COMMANDS_GENERATED.md
```

If there are differences in command signatures or options, update `COMMANDS.md` accordingly.

### `generate_command_docs.rs`

Rust version of the documentation generator (more robust).

**Usage:**

```bash
# Option 1: Run as binary (requires adding to Cargo.toml)
cargo run --bin generate_command_docs

# Option 2: Run as script (requires cargo-script)
cargo install cargo-script
cargo script scripts/generate_command_docs.rs
```

## Documentation Maintenance

### Two-Layer Documentation System

We maintain two types of command documentation:

1. **`COMMANDS.md` (Manual, Human-Friendly)**
   - Comprehensive examples with output
   - Common usage patterns
   - Best practices
   - JSON output examples
   - Quick reference sections
   - Detailed explanations
   
2. **`COMMANDS_GENERATED.md` (Auto-Generated, Verification)**
   - Raw CLI help text
   - Exact command signatures
   - Up-to-date option descriptions
   - Verification that docs match implementation

### Workflow

1. **When adding a new command:**
   ```bash
   # 1. Implement command with proper help text in code
   # 2. Build CLI
   cargo build --release
   
   # 3. Generate docs
   ./crates/cli/scripts/generate-command-docs.sh
   
   # 4. Compare
   diff crates/cli/docs/COMMANDS.md crates/cli/docs/COMMANDS_GENERATED.md
   
   # 5. Update COMMANDS.md manually with examples and patterns
   ```

2. **When updating command options:**
   ```bash
   # 1. Update code with new options
   # 2. Regenerate docs
   ./crates/cli/scripts/generate-command-docs.sh
   
   # 3. Check diff to see what changed
   diff crates/cli/docs/COMMANDS.md crates/cli/docs/COMMANDS_GENERATED.md
   
   # 4. Update COMMANDS.md with new options and examples
   ```

3. **Regular sync check:**
   ```bash
   # Run in CI/CD to verify docs are up-to-date
   ./crates/cli/scripts/generate-command-docs.sh
   diff crates/cli/docs/COMMANDS.md crates/cli/docs/COMMANDS_GENERATED.md
   
   # If diff is significant, update COMMANDS.md
   ```

## CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/docs-check.yml
name: Documentation Check

on: [pull_request]

jobs:
  check-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build CLI
        run: cargo build --release
      
      - name: Generate command docs
        run: ./crates/cli/scripts/generate-command-docs.sh
      
      - name: Check for changes
        run: |
          if ! diff -q crates/cli/docs/COMMANDS.md crates/cli/docs/COMMANDS_GENERATED.md; then
            echo "⚠️  Command documentation may be out of sync"
            echo "Review diff and update COMMANDS.md if needed"
            diff crates/cli/docs/COMMANDS.md crates/cli/docs/COMMANDS_GENERATED.md || true
          fi
```

## Best Practices

### Help Text in Code

Write clear, comprehensive help text in your command definitions:

```rust
/// Initialize project configuration.
///
/// Creates a new configuration file for changeset-based version management.
/// Supports interactive and non-interactive modes.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Changeset directory path.
    ///
    /// Directory where changeset files will be stored.
    ///
    /// Default: .changesets/
    #[arg(long, value_name = "PATH", default_value = ".changesets")]
    pub changeset_path: PathBuf,
}
```

This help text will:
- Appear in `--help` output
- Be extracted by generation scripts
- Provide documentation to users

### Manual Documentation

In `COMMANDS.md`, add:
- **Real examples** with actual command execution
- **Output samples** in both human and JSON formats
- **Common patterns** showing typical workflows
- **Tips and tricks** for power users
- **Troubleshooting** for common issues

## Troubleshooting

### Binary not found

```bash
# Error: CLI binary not found
# Solution: Build the CLI first
cargo build --release
```

### Permission denied

```bash
# Error: Permission denied: ./scripts/generate-command-docs.sh
# Solution: Make script executable
chmod +x crates/cli/scripts/generate-command-docs.sh
```

### Help text incomplete

```bash
# Issue: Generated docs missing information
# Solution: Ensure all commands have proper doc comments
# in the code using /// and #[arg(help = "...")]
```

## Files

```
crates/cli/
├── docs/
│   ├── COMMANDS.md              # Manual, human-friendly
│   ├── COMMANDS_GENERATED.md    # Auto-generated from help
│   └── GUIDE.md                 # User guide
└── scripts/
    ├── README.md                # This file
    ├── generate-command-docs.sh # Shell script generator
    └── generate_command_docs.rs # Rust script generator
```
