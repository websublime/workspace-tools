# Registry Override in Upgrade Check - Implementation Plan

**Version**: 1.0  
**Date**: 2025-01-20  
**Status**: Planning Complete  
**Author**: Development Team

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current System State](#current-system-state)
3. [Problem Analysis](#problem-analysis)
4. [Proposed Architecture](#proposed-architecture)
5. [Implementation Details](#implementation-details)
6. [Use Cases](#use-cases)
7. [Implementation Checklist](#implementation-checklist)
8. [Risks and Mitigations](#risks-and-mitigations)
9. [Summary](#summary)

---

## Executive Summary

### Problem Statement

The `workspace upgrade check` command currently ignores the `--registry` flag, which exists in the CLI arguments but has no implementation. This prevents users from:

1. Checking upgrades against private/custom NPM registries
2. Testing against different registry endpoints
3. Using organization-specific registries
4. Working behind corporate proxies with custom registries

**Current Behavior:**
```bash
workspace upgrade check --registry https://custom-registry.example.com
# ❌ Flag is silently ignored
# ✅ Always uses registry from workspace configuration
```

**Expected Behavior:**
```bash
workspace upgrade check --registry https://custom-registry.example.com
# ✅ Queries custom-registry.example.com for package versions
# ✅ Overrides workspace configuration registry
# ✅ Returns available upgrades from custom registry
```

### Solution Overview

Implement registry override with support for:

✅ **URL Override**: Specify any valid NPM registry URL  
✅ **Configuration Priority**: CLI flag overrides workspace config  
✅ **Validation**: URL format and connectivity validation  
✅ **Authentication**: Support for authenticated registries  
✅ **Backward Compatible**: No changes when flag is not used  

---

## Current System State

### 2.1 CLI Argument Exists But Is Unused

**File**: `crates/cli/src/cli/commands.rs:503`

```rust
#[derive(Debug, Args)]
pub struct UpgradeCheckArgs {
    // ... other fields ...
    
    /// Override registry URL.
    ///
    /// Uses this registry instead of the configured one.
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,  // ❌ Defined but NEVER used!
    
    // ... other fields ...
}
```

**Usage in Implementation** (`crates/cli/src/commands/upgrade/check.rs`):

```rust
pub async fn execute_upgrade_check(
    args: &UpgradeCheckArgs,  // ← args.registry exists here
    output: &Output,
    root: &Path,
) -> Result<()> {
    // Load configuration
    let config = load_config(root).await?;
    
    // Create upgrade checker
    let checker = UpgradeChecker::new(
        root.to_path_buf(),
        config.clone(),  // ✅ Uses config registry
        // ❌ args.registry is NEVER passed or used!
    ).await?;
    
    // ... rest of code ...
}
```

### 2.2 UpgradeChecker Infrastructure

**File**: `crates/pkg/src/upgrade/checker.rs`

**Current Constructor:**

```rust
impl UpgradeChecker {
    /// Creates a new upgrade checker.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Workspace root directory
    /// * `config` - Package tools configuration (contains registry URL)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let checker = UpgradeChecker::new(
    ///     workspace_root,
    ///     config,  // Registry URL from config
    /// ).await?;
    /// ```
    pub async fn new(
        workspace_root: PathBuf,
        config: PackageToolsConfig,
    ) -> UpgradeResult<Self> {
        let registry_url = config.npm.registry.clone();
        // Uses registry from config
        // ❌ No way to override!
    }
}
```

**Registry Client** (`crates/pkg/src/npm/client.rs`):

```rust
pub struct NpmClient {
    registry_url: String,
    client: reqwest::Client,
}

impl NpmClient {
    /// Creates a new NPM client.
    ///
    /// # Arguments
    ///
    /// * `registry_url` - NPM registry URL
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::npm::NpmClient;
    ///
    /// let client = NpmClient::new("https://registry.npmjs.org");
    /// ```
    pub fn new(registry_url: impl Into<String>) -> Self {
        Self {
            registry_url: registry_url.into(),
            client: reqwest::Client::new(),
        }
    }
}
```

**Good News**: Infrastructure already supports custom registries!

✅ `NpmClient` accepts any registry URL  
✅ No hardcoded registry endpoints  
✅ Only need to pass CLI override through the chain  

### 2.3 Configuration Structure

**File**: `crates/pkg/src/config/mod.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmConfig {
    /// NPM registry URL.
    ///
    /// Default: <https://registry.npmjs.org>
    #[serde(default = "default_registry")]
    pub registry: String,
}

fn default_registry() -> String {
    "https://registry.npmjs.org".to_string()
}
```

**Registry Source Priority:**

1. CLI flag `--registry` (❌ not implemented)
2. Workspace configuration file (✅ current behavior)
3. Default `https://registry.npmjs.org` (✅ fallback)

---

## Problem Analysis

### 3.1 Core Questions

**Q1: Where should the override be applied?**

**A**: In `execute_upgrade_check()` before creating `UpgradeChecker`.

**Q2: Should it modify the config or be passed separately?**

**A**: Clone config and modify registry value (simpler, clearer).

**Q3: Should it validate the registry URL?**

**A**: Yes - validate format and optionally test connectivity.

**Q4: What about authentication?**

**A**: Support later via `--registry-token` flag (future enhancement).

**Q5: Should it work with `upgrade apply`?**

**A**: No - only affects `check` command (avoid accidental installs from wrong registry).

### 3.2 Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Override Method** | Clone config + modify | Simple, no API changes needed |
| **Validation** | URL format only | Avoid blocking on network issues |
| **Authentication** | Future enhancement | Keep initial implementation simple |
| **Apply Command** | Not supported | Safety - avoid wrong registry installs |
| **Error Handling** | Clear, helpful errors | Guide users to fix issues |
| **Logging** | Debug log shows override | Transparency for troubleshooting |

---

## Proposed Architecture

### 4.1 High-Level Flow

```
┌──────────────────────────────────────────────────────────────┐
│  CLI: workspace upgrade check --registry https://custom.com  │
└─────────────────────┬────────────────────────────────────────┘
                      │
                      ↓
        ┌─────────────────────────────┐
        │  Parse --registry argument   │
        │  "https://custom.com"        │
        └──────────┬──────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Validate URL Format         │
        │  - Must be valid HTTP(S) URL │
        │  - Must not have trailing /  │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Load Workspace Config       │
        │  registry: registry.npmjs.org│
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Override Registry           │
        │  config.npm.registry =       │
        │    "https://custom.com"      │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Create UpgradeChecker       │
        │  with modified config        │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Check Upgrades              │
        │  Query custom registry       │
        └──────────────────────────────┘
```

### 4.2 Validation Helper

```rust
/// Validates and normalizes a registry URL.
///
/// # What
///
/// Validates that a registry URL is properly formatted and normalizes it
/// by removing trailing slashes and ensuring HTTPS (when possible).
///
/// # Why
///
/// Ensures registry URLs are valid before attempting to use them, preventing
/// obscure HTTP errors and providing clear feedback to users.
///
/// # Arguments
///
/// * `url` - Registry URL to validate
///
/// # Returns
///
/// Normalized registry URL.
///
/// # Errors
///
/// Returns error if:
/// - URL is not a valid HTTP/HTTPS URL
/// - URL contains invalid characters
/// - URL scheme is not http or https
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::validate_registry_url;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let url = validate_registry_url("https://custom.com/")?;
/// assert_eq!(url, "https://custom.com");
///
/// let url = validate_registry_url("https://registry.npmjs.org")?;
/// assert_eq!(url, "https://registry.npmjs.org");
/// # Ok(())
/// # }
/// ```
pub fn validate_registry_url(url: &str) -> Result<String> {
    // Parse URL
    let parsed = url::Url::parse(url).map_err(|e| {
        CliError::validation(format!(
            "Invalid registry URL '{}': {}",
            url, e
        ))
    })?;

    // Validate scheme
    match parsed.scheme() {
        "http" | "https" => {},
        scheme => {
            return Err(CliError::validation(format!(
                "Registry URL must use HTTP or HTTPS scheme, found: {}",
                scheme
            )));
        }
    }

    // Validate host exists
    if parsed.host_str().is_none() {
        return Err(CliError::validation(format!(
            "Registry URL must have a valid host: {}",
            url
        )));
    }

    // Remove trailing slash for consistency
    let normalized = url.trim_end_matches('/').to_string();

    Ok(normalized)
}
```

---

## Implementation Details

### 5.1 Modify Upgrade Check Command

**File**: `crates/cli/src/commands/upgrade/check.rs`

**Add registry override:**

```rust
pub async fn execute_upgrade_check(
    args: &UpgradeCheckArgs,
    output: &Output,
    root: &Path,
) -> Result<()> {
    let workspace_root = root;
    debug!("Checking for available upgrades in workspace: {}", workspace_root.display());

    // Step 1: Load workspace configuration
    let mut config = load_config(workspace_root).await?;
    info!("Configuration loaded successfully");

    // ✅ Step 2: Override registry if specified
    if let Some(ref registry_url) = args.registry {
        debug!("Registry override requested: {}", registry_url);
        
        // Validate URL format
        let validated_url = validate_registry_url(registry_url)?;
        
        // Override config registry
        let original_registry = config.npm.registry.clone();
        config.npm.registry = validated_url.clone();
        
        info!(
            "Registry override applied: {} → {}",
            original_registry,
            validated_url
        );
        
        // Inform user in human mode
        if !output.format().is_json() {
            output.info(&format!(
                "Using custom registry: {}",
                validated_url
            ))?;
        }
    }

    // Step 3: Create upgrade checker (now uses potentially overridden config)
    let checker = UpgradeChecker::new(
        workspace_root.to_path_buf(),
        config.clone(),  // ✅ Config may have overridden registry
    )
    .await
    .map_err(|e| CliError::execution(format!("Failed to create upgrade checker: {e}")))?;

    // Step 4: Determine which dependency types to check
    let check_dev = !args.no_dev;
    let check_peer = args.peer;

    debug!("Checking dependency types - dev: {}, peer: {}", check_dev, check_peer);

    // Step 5: Check for upgrades with filters
    let all_upgrades = checker
        .check_all()
        .await
        .map_err(|e| CliError::execution(format!("Failed to check for upgrades: {e}")))?;

    debug!("Found {} total upgrade(s)", all_upgrades.len());

    // ... rest of existing filtering and output code ...

    Ok(())
}
```

### 5.2 Add Validation Function

**File**: `crates/cli/src/commands/upgrade/check.rs`

Add the `validate_registry_url()` function from section 4.2.

### 5.3 Add url Dependency

**File**: `crates/cli/Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...
url = "2.5"  # For URL parsing and validation
```

### 5.4 Update Help Text

**File**: `crates/cli/src/cli/commands.rs`

```rust
#[derive(Debug, Args)]
pub struct UpgradeCheckArgs {
    // ... other fields ...

    /// Override registry URL.
    ///
    /// Uses this registry instead of the configured one.
    /// Must be a valid HTTP or HTTPS URL.
    ///
    /// Examples:
    /// - https://custom-registry.example.com
    /// - https://registry.npmjs.org
    /// - http://localhost:4873 (Verdaccio local registry)
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,

    // ... other fields ...
}
```

---

## Use Cases

### 6.1 Private Company Registry

**Scenario**: Company uses private NPM registry for internal packages.

```bash
# Default registry doesn't have company packages
workspace upgrade check
# ❌ Doesn't find upgrades for @company/* packages

# Use company registry
workspace upgrade check --registry https://npm.company.com
# ✅ Finds upgrades for both public and @company/* packages
```

### 6.2 Testing Against Different Registries

**Scenario**: Compare versions available in different registries.

```bash
# Check what's in npmjs
workspace upgrade check --registry https://registry.npmjs.org > npmjs.txt

# Check what's in GitHub packages
workspace upgrade check --registry https://npm.pkg.github.com > github.txt

# Compare
diff npmjs.txt github.txt
```

### 6.3 Local Registry Development

**Scenario**: Testing with local Verdaccio registry.

```bash
# Start local Verdaccio
verdaccio

# Check against local registry
workspace upgrade check --registry http://localhost:4873
# ✅ Checks against local mirror
```

### 6.4 Corporate Proxy

**Scenario**: Corporate network routes all NPM traffic through proxy.

```bash
# Use corporate proxy registry
workspace upgrade check --registry https://npm-proxy.corp.internal
# ✅ Works behind corporate firewall
```

### 6.5 JSON Output for Automation

**Scenario**: CI pipeline checks multiple registries.

```bash
#!/bin/bash

registries=(
  "https://registry.npmjs.org"
  "https://npm.company.com"
  "https://npm.pkg.github.com"
)

for registry in "${registries[@]}"; do
  echo "Checking $registry..."
  workspace upgrade check \
    --registry "$registry" \
    --format json > "upgrades-$(echo $registry | sed 's/[^a-z]//g').json"
done
```

---

## Implementation Checklist

### Phase 1: Core Implementation
- [ ] Add `url` dependency to `crates/cli/Cargo.toml`
- [ ] Implement `validate_registry_url()` in `check.rs`
- [ ] Modify `execute_upgrade_check()` to apply registry override
- [ ] Unit tests for `validate_registry_url()`
- [ ] Unit tests for valid URLs (http, https)
- [ ] Unit tests for invalid URLs (ftp, invalid format, etc.)
- [ ] Unit tests for URL normalization (trailing slash removal)

### Phase 2: Integration
- [ ] E2E test: check with custom registry URL
- [ ] E2E test: check with http://localhost registry
- [ ] E2E test: invalid registry URL (should error)
- [ ] E2E test: default behavior (no --registry flag)
- [ ] Verify registry client receives overridden URL
- [ ] Check debug logs show registry override

### Phase 3: User Experience
- [ ] Update help text with examples
- [ ] Add info message showing registry being used (human mode)
- [ ] Test error messages are clear and helpful
- [ ] JSON output includes registry information
- [ ] Verify color output works correctly
- [ ] Test quiet mode behavior

### Phase 4: Edge Cases
- [ ] Test with registry URL containing port (http://localhost:4873)
- [ ] Test with registry URL containing path (/registry/)
- [ ] Test with invalid schemes (ftp://, file://)
- [ ] Test with malformed URLs
- [ ] Test with empty string
- [ ] Test with very long URLs
- [ ] Test with URLs containing special characters

### Phase 5: Documentation
- [ ] Update `crates/cli/SPEC.md` with registry override
- [ ] Update CLI help text
- [ ] Add examples to README
- [ ] Document common registry URLs
- [ ] Add troubleshooting section
- [ ] Update CHANGELOG

### Phase 6: Final Validation
- [ ] All unit tests passing
- [ ] All E2E tests passing
- [ ] Clippy warnings resolved
- [ ] Format check passing
- [ ] Manual testing with real registries
- [ ] Test with private registry
- [ ] Test with local Verdaccio
- [ ] Code review completed

---

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|---------|-----------|
| Invalid URL crashes tool | Low | Medium | URL validation before use |
| Network timeout issues | Medium | Low | Existing timeout handling in NpmClient |
| Authentication not supported | High | Medium | Document limitation, plan future enhancement |
| Confusion about which registry used | Medium | Medium | Log and display registry in use |
| Apply command using wrong registry | Low | High | Don't implement for `apply` command |
| Corporate proxy SSL issues | Medium | Low | Document as known limitation |
| Registry URL typo | High | Low | Validation catches most errors |
| Performance impact | Very Low | Very Low | No additional overhead |

---

## Summary

### 9.1 Key Features

✅ **URL Override**: Any valid HTTP/HTTPS registry URL  
✅ **Validation**: Format validation before use  
✅ **Transparency**: Clear logging of registry in use  
✅ **Backward Compatible**: No changes when flag not used  
✅ **Safety First**: Only affects check, not apply  
✅ **Simple**: Minimal code changes, leverages existing infrastructure  

### 9.2 Future Enhancements

**Not in Initial Implementation:**
- Authentication support (`--registry-token`)
- Connectivity testing
- Registry alias shortcuts
- Multiple registry checking in one command

**Rationale**: Keep initial implementation simple and focused. Add features based on user feedback.

### 9.3 Success Metrics

- [ ] Works with all documented use cases
- [ ] Clear error messages for all failure scenarios
- [ ] 100% test coverage for new code
- [ ] Zero clippy warnings
- [ ] Complete documentation
- [ ] Manual validation with different registries
- [ ] Team approval

### 9.4 Implementation Timeline

**Estimated Effort**: 0.5-1 day

- **Phase 1**: Core Implementation (0.25 day)
- **Phase 2**: Integration (0.25 day)
- **Phase 3**: User Experience (0.25 day)
- **Phase 4**: Edge Cases (0.25 day)
- **Phase 5**: Documentation (0.25 day)
- **Phase 6**: Final Validation (0.25 day)

---

## Next Steps

1. **Review plan** with team
2. **Approve approach** (config clone vs separate parameter)
3. **Begin Phase 1** implementation
4. **Test with real registries** during development
5. **Document** as you go
6. **Validate** with different registry types

---

**Status**: Ready for Implementation 🚀

This solution provides **simple, safe registry override** for the upgrade check command with minimal code changes and maximum clarity.
