# Migration Guide: Sublime Monorepo Tools v0.1.0

This guide helps you migrate from the previous complex API to the new streamlined CLI/daemon-focused API.

## 🎯 Overview

The refactoring reduced the public API from **20 types to 12 types** while focusing on CLI and daemon consumption patterns. The new API emphasizes direct ownership patterns, sub-second performance, and simplified architecture.

## 📊 Summary of Changes

### API Reduction
- **Before**: 20 public types across 13 modules
- **After**: 12 public types across 8 modules  
- **Reduction**: 40% smaller API surface

### Performance Improvements
- **Startup Time**: < 100ms (CLI responsiveness)
- **Analysis Time**: < 1s (real-time feedback)
- **Memory Usage**: Efficient direct borrowing patterns

### Architecture Changes
- Removed service abstractions in favor of direct base crate usage
- Eliminated Arc proliferation for better performance
- Streamlined for CLI/daemon consumption

---

## 🔴 Breaking Changes Summary

### Removed Features
- **Workflows Module**: Complex orchestration removed
- **Plugins System**: Over-engineered plugin architecture removed  
- **Event Bus**: Event-driven architecture removed
- **Complex Hooks**: Abstracted git hook system removed
- **MonorepoTools**: Replaced with direct component access
- **MonorepoServices**: Service container pattern removed

### API Changes
- **Public API**: Reduced from 20 to 12 types
- **Direct Access**: Services replaced with direct field access
- **Simplified Constructors**: No more complex service initialization

---

## 🟢 New API (12 Types)

### Core Project Management (1 type)
```rust
pub use crate::core::MonorepoProject;
```

### Analysis (2 types)  
```rust
pub use crate::analysis::{MonorepoAnalyzer, ChangeAnalysis};
```

### Configuration (3 types)
```rust
pub use crate::config::{Environment, MonorepoConfig, VersionBumpType};
```

### Version Management (2 types)
```rust
pub use crate::core::{VersionManager, VersioningResult};
```

### Change Detection (2 types)
```rust
pub use crate::changes::{ChangeDetector, PackageChange};
```

### Error Handling (2 types)
```rust
pub use crate::error::{Error, Result};
```

---

## 📚 Migration Examples

### 1. Project Initialization

**Before:**
```rust
// Complex service container initialization
let services = MonorepoServices::new("/path/to/monorepo")?;
let project = MonorepoProject::from_services(services)?;
let tools = MonorepoTools::new(&project);
```

**After:**
```rust
// Direct initialization with base crate integration
let project = MonorepoProject::new("/path/to/monorepo")?;
```

### 2. Package Analysis

**Before:**
```rust
let tools = MonorepoTools::new(&project);
let analyzer = tools.analyzer()?;
let packages = analyzer.get_packages();
```

**After:**
```rust
// Direct analyzer creation
let analyzer = MonorepoAnalyzer::new(&project);
let packages = analyzer.get_packages();
```

### 3. Version Management

**Before:**
```rust
let tools = MonorepoTools::new(&project);
let version_manager = tools.version_manager();
let result = version_manager.bump_package_version("package", VersionBumpType::Minor, None)?;
```

**After:**
```rust
// Direct version manager creation
let version_manager = VersionManager::new(&project);
let result = version_manager.bump_package_version("package", VersionBumpType::Minor, None)?;
```

### 4. Change Detection

**Before:**
```rust
let analyzer = tools.analyzer()?;
let changes = analyzer.detect_changes_since("HEAD~1", None)?;
```

**After:**
```rust
// Same interface, simpler construction
let analyzer = MonorepoAnalyzer::new(&project);
let changes = analyzer.detect_changes_since("HEAD~1", None)?;
```

### 5. Configuration Access

**Before:**
```rust
let config_service = services.config_service();
let config = config_service.get_configuration();
```

**After:**
```rust
// Direct field access
let config = project.config();
```

### 6. File System Operations

**Before:**
```rust
let fs_service = services.file_system_service();
let root_path = fs_service.root_path();
```

**After:**
```rust
// Direct field access to base crates
let root_path = project.root_path();
let file_system = &project.file_system;
```

---

## 🔄 Feature Migration

### Workflows → CLI Commands
**Removed**: Complex workflow orchestration
**Alternative**: Implement workflow logic in CLI commands

**Before:**
```rust
let release_workflow = ReleaseWorkflow::new(&project);
let result = release_workflow.execute(ReleaseOptions::default())?;
```

**After:**
```rust
// Implement in CLI command
let version_manager = VersionManager::new(&project);
let analyzer = MonorepoAnalyzer::new(&project);
// Custom CLI logic using core components
```

### Plugin System → Direct Integration
**Removed**: Complex plugin architecture
**Alternative**: Direct integration with base crates

**Before:**
```rust
let plugin_manager = tools.plugin_manager();
plugin_manager.register_plugin(Box::new(MyPlugin::new()))?;
```

**After:**
```rust
// Use base crates directly
use sublime_standard_tools::commands::CommandExecutor;
use sublime_package_tools::Package;
// Direct implementation
```

### Event System → Direct Calls
**Removed**: Event bus architecture
**Alternative**: Direct method calls

**Before:**
```rust
event_bus.emit(PackageUpdatedEvent::new("package-name"))?;
```

**After:**
```rust
// Direct method calls
version_manager.bump_package_version("package-name", VersionBumpType::Patch, None)?;
```

### Hook System → Script Generation
**Removed**: Complex hook abstraction
**Alternative**: Simple script generation

**Before:**
```rust
let hook_manager = tools.hook_manager();
hook_manager.install_hook(HookType::PreCommit, hook_config)?;
```

**After:**
```rust
// Generate simple git hook scripts
let pre_commit_script = r#"#!/bin/sh
cargo test --lib
"#;
std::fs::write(".git/hooks/pre-commit", pre_commit_script)?;
```

---

## 🏗️ Architecture Migration

### Service Container → Direct Access
**Before:**
```rust
struct MyComponent {
    services: Arc<MonorepoServices>,
}

impl MyComponent {
    fn new(services: Arc<MonorepoServices>) -> Self {
        Self { services }
    }
    
    fn do_work(&self) -> Result<()> {
        let config = self.services.config_service().get_configuration();
        let packages = self.services.package_service().discover_packages()?;
        // ...
    }
}
```

**After:**
```rust
struct MyComponent<'a> {
    project: &'a MonorepoProject,
}

impl<'a> MyComponent<'a> {
    fn new(project: &'a MonorepoProject) -> Self {
        Self { project }
    }
    
    fn do_work(&self) -> Result<()> {
        let config = self.project.config();
        let packages = &self.project.packages;
        // Direct field access, no Arc overhead
    }
}
```

### Async → Sync
**Removed**: Async complexity
**Benefit**: Simpler execution model for CLI

**Before:**
```rust
async fn analyze_project() -> Result<ChangeAnalysis> {
    let analyzer = tools.analyzer().await?;
    analyzer.detect_changes_since("HEAD~1", None).await
}
```

**After:**
```rust
fn analyze_project() -> Result<ChangeAnalysis> {
    let analyzer = MonorepoAnalyzer::new(&project);
    analyzer.detect_changes_since("HEAD~1", None)
}
```

---

## 📈 Performance Improvements

### Startup Time
- **Before**: Variable startup time due to service initialization
- **After**: < 100ms consistent startup for CLI responsiveness

### Memory Usage
- **Before**: Arc proliferation and service overhead
- **After**: Direct borrowing patterns, minimal allocations

### Analysis Speed
- **Before**: Multiple abstraction layers
- **After**: < 1s analysis time with direct base crate access

### Dependency Graph
- **Before**: O(n²) dependency graph building
- **After**: O(n) with HashMap optimization

---

## ✅ Migration Checklist

### 1. Update Dependencies
```toml
[dependencies]
sublime_monorepo_tools = "0.1.0"
```

### 2. Replace Service Container Usage
- [ ] Remove `MonorepoServices::new()` calls
- [ ] Replace with `MonorepoProject::new()`
- [ ] Update service access to direct field access

### 3. Update Tool Creation
- [ ] Remove `MonorepoTools::new()` calls
- [ ] Create components directly:
  - `MonorepoAnalyzer::new(&project)`
  - `VersionManager::new(&project)`

### 4. Simplify Configuration
- [ ] Replace `services.config_service().get_configuration()`
- [ ] Use `project.config()`

### 5. Update Workflows
- [ ] Remove workflow module usage
- [ ] Implement logic using core components
- [ ] Use CLI commands for orchestration

### 6. Replace Plugins
- [ ] Remove plugin system usage
- [ ] Use base crates directly
- [ ] Implement functionality in CLI

### 7. Remove Event Handling
- [ ] Replace event emissions with direct calls
- [ ] Remove event listeners
- [ ] Use synchronous method calls

### 8. Simplify Git Hooks
- [ ] Remove complex hook abstractions
- [ ] Generate simple shell scripts
- [ ] Use git hooks directly

---

## 🔧 Common Migration Patterns

### Pattern 1: Service Access → Direct Access
```rust
// Before
let file_service = services.file_system_service();
let exists = file_service.exists(&path)?;

// After  
use sublime_standard_tools::filesystem::FileSystem;
let exists = project.file_system.exists(&path);
```

### Pattern 2: Tool Factory → Direct Construction
```rust
// Before
let tools = MonorepoTools::new(&project);
let analyzer = tools.analyzer()?;

// After
let analyzer = MonorepoAnalyzer::new(&project);
```

### Pattern 3: Complex Initialization → Simple Constructor
```rust
// Before
let services = MonorepoServices::new(root_path)?;
let project = MonorepoProject::from_services(services)?;

// After
let project = MonorepoProject::new(root_path)?;
```

---

## 🐛 Troubleshooting

### Issue: "MonorepoTools not found"
**Solution**: Replace with direct component creation
```rust
// Replace this
let tools = MonorepoTools::new(&project);

// With this
let analyzer = MonorepoAnalyzer::new(&project);
let version_manager = VersionManager::new(&project);
```

### Issue: "Service not available"
**Solution**: Use direct field access
```rust
// Replace this
let config = services.config_service().get_configuration();

// With this
let config = project.config();
```

### Issue: "Plugin system removed"
**Solution**: Use base crates directly
```rust
// Replace plugin usage with
use sublime_standard_tools::commands::CommandExecutor;
use sublime_package_tools::Package;
```

### Issue: "Async function expected"
**Solution**: Remove async/await
```rust
// Replace this
let result = analyzer.analyze().await?;

// With this
let result = analyzer.analyze()?;
```

---

## 🎉 Benefits After Migration

### Simplified API
- 40% fewer public types to learn
- Consistent patterns across all operations
- Direct access without service layers

### Better Performance
- < 100ms startup time
- < 1s analysis operations
- Efficient memory usage

### Easier Debugging
- Direct method calls instead of event chains
- Clear ownership patterns
- Simplified stack traces

### CLI/Daemon Ready
- Optimized for CLI responsiveness
- Daemon-friendly architecture
- Minimal resource usage

---

## 📞 Support

If you encounter issues during migration:

1. **Check this guide** for common patterns
2. **Review the examples** in the README.md
3. **Check the documentation** for specific types
4. **File an issue** if you find missing functionality

The new API is designed to be simpler and more direct. Most migration involves removing complexity rather than adding it.