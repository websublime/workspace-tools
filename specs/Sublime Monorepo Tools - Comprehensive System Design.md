# Sublime Monorepo Tools - Comprehensive System Design

## Overview

`sublime_monorepo_tools` is a comprehensive library for JavaScript/TypeScript monorepo management that integrates with the existing Workspace Node Tools ecosystem. It builds upon `sublime_standard_tools`, `sublime_git_tools`, and `sublime_package_tools` to provide a unified solution for monorepo workflow management.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Workspace Node Tools                        │
│                                                                 │
│  ┌─────────────────┐   ┌─────────────────┐  ┌─────────────────┐ │
│  │     Node.js     │   │     Node.js     │  │     Node.js     │ │
│  │    Bindings     │   │    Bindings     │  │    Bindings     │ │
│  └────────┬────────┘   └────────┬────────┘  └────────┬────────┘ │
│           │                     │                    │          │
│  ┌────────▼────────┐   ┌────────▼────────┐  ┌────────▼────────┐ │
│  │sublime_monorepo_│   │ sublime_package_│  │ sublime_git_    │ │
│  │     tools       │◄─►│      tools      │◄─►│     tools      │ │
│  └────────┬────────┘   └────────┬────────┘  └────────┬────────┘ │
│           │                     │                    │          │
│           └─────────────┬───────┴────────────────────┘          │
│                         │                                       │
│                ┌────────▼────────┐                              │
│                │sublime_standard_│                              │
│                │     tools       │                              │
│                └─────────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────────────────┐
│                   External Integrations                        │
│                                                                │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐    │
│  │  Git Hooks     │  │  GitHub Actions│  │  Turborepo     │    │
│  └────────────────┘  └────────────────┘  └────────────────┘    │
│                                                                │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐    │
│  │  CI Systems    │  │  Package Mgrs  │  │  Changelogs    │    │
│  └────────────────┘  └────────────────┘  └────────────────┘    │
└────────────────────────────────────────────────────────────────┘
```

## Core Module Structure

```
sublime_monorepo_tools/
├── Cargo.toml
└── src/
    ├── lib.rs                       # Main exports and types
    ├── workspace/                   # Workspace management
    │   ├── mod.rs
    │   ├── discovery.rs             # Package discovery
    │   ├── manifest.rs              # Workspace manifest handling
    │   └── graph.rs                 # Workspace dependency graph
    ├── changes/                     # Change management
    │   ├── mod.rs
    │   ├── tracker.rs               # Change tracking
    │   ├── changeset.rs             # Changeset implementation
    │   └── store.rs                 # Persistent storage
    ├── versioning/                  # Version management
    │   ├── mod.rs
    │   ├── bump.rs                  # Version bumping
    │   ├── suggest.rs               # Version suggestion
    │   └── changelog.rs             # Changelog generation
    ├── tasks/                       # Task execution
    │   ├── mod.rs
    │   ├── runner.rs                # Task runner
    │   ├── graph.rs                 # Task dependency graph
    │   └── parallel.rs              # Parallel execution
    ├── deployment/                  # Deployment tracking
    │   ├── mod.rs
    │   ├── environment.rs           # Environment definitions
    │   └── tracker.rs               # Deployment tracking
    ├── github/                      # GitHub integration
    │   ├── mod.rs
    │   ├── workflows.rs             # GitHub Actions workflows
    │   └── app.rs                   # GitHub App integration
    ├── hooks/                       # Git hooks integration
    │   ├── mod.rs
    │   ├── manager.rs               # Hook installation
    │   ├── pre_commit.rs            # Pre-commit hooks
    │   ├── post_commit.rs           # Post-commit hooks
    │   └── templates/               # Hook templates
    │       ├── pre_commit.sh        # Pre-commit template
    │       └── post_commit.sh       # Post-commit template
    ├── integration/                 # External tools integration
    │   ├── mod.rs
    │   ├── turbo.rs                 # Turborepo integration
    │   ├── nx.rs                    # Nx integration
    │   └── lerna.rs                 # Lerna compatibility
    └── utils/                       # Utility functions
        ├── mod.rs
        ├── fs.rs                    # Filesystem utilities
        └── napi.rs                  # NAPI binding helpers
```

## API Specification

### 1. Workspace Management

The primary interface for discovering, loading, and managing monorepo workspaces.

#### WorkspaceManager

```rust
/// Main entry point for workspace operations
pub struct WorkspaceManager {
    // Internal implementation
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new() -> Self;
    
    /// Discover workspace from directory
    pub fn discover_workspace(
        &self, 
        path: impl AsRef<Path>, 
        options: DiscoveryOptions
    ) -> Result<Workspace, WorkspaceError>;
    
    /// Load workspace from explicit configuration
    pub fn load_workspace(
        &self,
        config: WorkspaceConfig
    ) -> Result<Workspace, WorkspaceError>;
    
    /// Analyze workspace for issues
    pub fn analyze_workspace(
        &self,
        workspace: &Workspace,
    ) -> Result<WorkspaceAnalysis, WorkspaceError>;
}
```

#### Workspace

```rust
/// Complete workspace representation
pub struct Workspace {
    root_path: PathBuf,
    package_infos: Vec<Rc<RefCell<PackageInfo>>>,
    package_manager: Option<CorePackageManager>,
    git_repo: Option<Rc<Repo>>,
}

impl Workspace {
    /// Get a package by name
    pub fn get_package(&self, name: &str) -> Option<Rc<RefCell<PackageInfo>>>;
    
    /// Get packages in topological order
    pub fn sorted_packages(&self) -> Vec<Rc<RefCell<PackageInfo>>>;
    
    /// Get packages affected by changes
    pub fn affected_packages(&self, changed_packages: &[&str]) -> Vec<Rc<RefCell<PackageInfo>>>;
    
    /// Get packages that depend on a specific package
    pub fn dependents_of(&self, package_name: &str) -> Vec<Rc<RefCell<PackageInfo>>>;
    
    /// Get direct dependencies of a package
    pub fn dependencies_of(&self, package_name: &str) -> Vec<Rc<RefCell<PackageInfo>>>;
    
    /// Get workspace root path
    pub fn root_path(&self) -> &Path;
    
    /// Get Git repository reference
    pub fn git_repo(&self) -> Option<&Repo>;
    
    /// Get package manager
    pub fn package_manager(&self) -> &Option<CorePackageManager>;
    
    /// Write package changes to disk
    pub fn write_changes(&self) -> Result<(), WorkspaceError>;
    
    /// Validate workspace consistency
    pub fn validate(&self) -> Result<ValidationReport, WorkspaceError>;
    
    /// Build dependency graph from workspace packages
    pub fn build_dependency_graph(&self) -> Result<DependencyGraph<'_, Package>, WorkspaceError>;
}
```

### 2. Change Management

The changes module handles tracking, storing, and analyzing changes across packages.

#### ChangeTracker

```rust
/// Change tracking system
pub struct ChangeTracker {
    workspace: Rc<Workspace>,
    store: Box<dyn ChangeStore>,
}

impl ChangeTracker {
    /// Create a new change tracker
    pub fn new(
        workspace: Rc<Workspace>,
        store: Box<dyn ChangeStore>,
    ) -> Self;
    
    /// Detect changes between Git references
    pub fn detect_changes_between(
        &self,
        from_ref: &str,
        to_ref: Option<&str>,
    ) -> Result<Vec<Change>, ChangeError>;
    
    /// Record a change manually
    pub fn record_change(&mut self, change: Change) -> Result<(), ChangeError>;
    
    /// Create and record a changeset
    pub fn create_changeset(
        &mut self,
        summary: Option<String>,
        changes: Vec<Change>
    ) -> Result<Changeset, ChangeError>;
    
    /// Get unreleased changes for all packages
    pub fn unreleased_changes(&self) -> Result<HashMap<String, Vec<&Change>>, ChangeError>;
    
    /// Mark changes as released
    pub fn mark_released(
        &mut self,
        package: &str,
        version: &str,
        dry_run: bool,
    ) -> Result<(), ChangeError>;
}
```

#### Change and Changeset

```rust
/// A single change record
pub struct Change {
    /// Package name
    pub package: String,
    /// Change type (feature, fix, etc.)
    pub change_type: ChangeType,
    /// Description
    pub description: String,
    /// Whether this is breaking
    pub breaking: bool,
    /// Creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Author
    pub author: Option<String>,
    /// Related issues
    pub issues: Vec<String>,
    /// Release version (None if unreleased)
    pub release_version: Option<String>,
}

/// Collection of related changes
pub struct Changeset {
    /// Unique identifier
    pub id: String,
    /// Summary
    pub summary: Option<String>,
    /// Changes in this set
    pub changes: Vec<Change>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Types of changes
pub enum ChangeType {
    Feature,
    Fix,
    Documentation,
    Performance,
    Refactor,
    Test,
    Chore,
    Build,
    CI,
    Revert,
    Style,
    Custom(String),
}
```

### 3. Versioning Management

The versioning module handles version bumping, suggestions, and changelog generation.

#### VersionManager

```rust
/// Version management system
pub struct VersionManager<'a> {
    workspace: &'a Workspace,
    change_tracker: Option<&'a ChangeTracker>,
}

impl<'a> VersionManager<'a> {
    /// Create a new version manager
    pub fn new(
        workspace: &'a Workspace,
        change_tracker: Option<&'a ChangeTracker>
    ) -> Self;
    
    /// Suggest version bumps based on changes
    pub fn suggest_bumps(
        &self,
        strategy: &VersionBumpStrategy
    ) -> Result<HashMap<String, VersionSuggestion>, VersionError>;
    
    /// Preview version bumps without applying
    pub fn preview_bumps(
        &self,
        strategy: &VersionBumpStrategy
    ) -> Result<VersionBumpPreview, VersionError>;
    
    /// Apply version bumps
    pub fn apply_bumps(
        &self,
        strategy: &VersionBumpStrategy,
        dry_run: bool
    ) -> Result<Vec<PackageVersionChange>, VersionError>;
    
    /// Generate changelogs
    pub fn generate_changelogs(
        &self,
        options: &ChangelogOptions,
        dry_run: bool
    ) -> Result<HashMap<String, String>, VersionError>;
    
    /// Validate version consistency
    pub fn validate_versions(&self) -> Result<VersionValidation, VersionError>;
}
```

#### Version Strategies

```rust
/// Strategy for version bumping
pub enum VersionBumpStrategy {
    /// All packages get the same version
    Synchronized { version: String },
    /// Each package bumped according to its changes
    Independent {
        major_if_breaking: bool,
        minor_if_feature: bool,
        patch_otherwise: bool,
    },
    /// Use conventional commit messages
    ConventionalCommits { from_ref: Option<String> },
    /// Manually specified versions
    Manual(HashMap<String, String>),
}

/// Version bump suggestion
pub struct VersionSuggestion {
    /// Package name
    pub package_name: String,
    /// Current version
    pub current_version: String,
    /// Suggested next version
    pub suggested_version: String,
    /// Type of bump
    pub bump_type: BumpType,
    /// Reasons for bump
    pub reasons: Vec<BumpReason>,
}
```

### 4. Task Management

The tasks module handles execution of commands across the monorepo with proper ordering and parallelism.

#### TaskRunner

```rust
/// Task execution system
pub struct TaskRunner<'a> {
    workspace: &'a Workspace,
    tasks: HashMap<String, Task>,
    max_parallelism: usize,
}

impl<'a> TaskRunner<'a> {
    /// Create a new task runner
    pub fn new(
        workspace: &'a Workspace,
        max_parallelism: Option<usize>
    ) -> Self;
    
    /// Add a task to the runner
    pub fn add_task(&mut self, task: Task) -> &mut Self;
    
    /// Run all tasks
    pub fn run_all(&self) -> Result<Vec<TaskResult>, TaskError>;
    
    /// Run specified tasks
    pub fn run_tasks(
        &self,
        task_names: &[&str]
    ) -> Result<Vec<TaskResult>, TaskError>;
    
    /// Run tasks matching filter
    pub fn run_filtered(
        &self,
        filter: TaskFilter
    ) -> Result<Vec<TaskResult>, TaskError>;
    
    /// Build task graph for visualization
    pub fn build_task_graph(&self) -> Result<TaskGraph, TaskError>;
}
```

#### Task Definitions

```rust
/// Definition of a task
pub struct Task {
    /// Task name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Package context
    pub package: Option<String>,
    /// Task dependencies
    pub dependencies: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory
    pub cwd: Option<PathBuf>,
    /// Timeout
    pub timeout: Option<Duration>,
}

/// Result of task execution
pub struct TaskResult {
    /// Original task
    pub task: Task,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    pub duration: Duration,
    /// Success flag
    pub success: bool,
}
```

### 5. Git Hooks Integration

The hooks module manages Git hooks for seamless workflow integration.

```rust
/// Git hooks management system
pub struct HooksManager<'a> {
    workspace: &'a Workspace,
}

impl<'a> HooksManager<'a> {
    /// Create a new hooks manager
    pub fn new(workspace: &'a Workspace) -> Self;
    
    /// Check if hooks are installed
    pub fn are_hooks_installed(&self) -> Result<HashMap<HookType, bool>, HookError>;
    
    /// Install Git hooks
    pub fn install_hooks(
        &self,
        hooks: &[HookType],
        options: &HookInstallOptions
    ) -> Result<(), HookError>;
    
    /// Uninstall Git hooks
    pub fn uninstall_hooks(&self, hooks: &[HookType]) -> Result<(), HookError>;
    
    /// Run a specific hook manually
    pub fn run_hook(
        &self,
        hook_type: HookType
    ) -> Result<HookResult, HookError>;
}
```

### 6. External Tool Integration

```rust
/// Turborepo integration
pub struct TurboIntegration<'a> {
    workspace: &'a Workspace,
}

impl<'a> TurboIntegration<'a> {
    /// Create new Turborepo integration
    pub fn new(workspace: &'a Workspace) -> Self;
    
    /// Generate turbo.json from workspace
    pub fn generate_config(&self) -> Result<TurboConfig, IntegrationError>;
    
    /// Write turbo.json
    pub fn write_config(
        &self,
        config: &TurboConfig,
        path: Option<&Path>
    ) -> Result<PathBuf, IntegrationError>;
    
    /// Optimize existing turbo.json
    pub fn optimize_config(
        &self,
        config: &TurboConfig
    ) -> Result<TurboConfig, IntegrationError>;
}
```

### 7. Node.js Bindings

```typescript
// TypeScript declarations for Node.js bindings
declare module "sublime_monorepo_tools" {
  export interface Workspace {
    rootPath: string;
    packages: Package[];
    packageManager?: "npm" | "yarn" | "pnpm" | "bun";
    
    getPackage(name: string): Package | null;
    getAffectedPackages(changedPackages: string[]): Package[];
    writeChanges(): Promise<void>;
    validate(): Promise<ValidationReport>;
  }
  
  export interface Package {
    name: string;
    version: string;
    location: string;
    dependencies: Record<string, string>;
    devDependencies: Record<string, string>;
  }
  
  // Main API
  export function discoverWorkspace(path?: string): Promise<Workspace>;
  export function recordChange(change: Change): Promise<void>;
  export function suggestVersionBumps(strategy: object): Promise<Record<string, object>>;
  export function applyVersionBumps(strategy: object, dryRun?: boolean): Promise<VersionBump[]>;
  export function generateChangelogs(options?: object): Promise<Record<string, string>>;
  export function installHooks(types: string[]): Promise<void>;
  export function runTasks(tasks: string[], filter?: object): Promise<object[]>;
}
```

## Integration with Existing Crates

### Integration with sublime\_standard\_tools

```rust
// Workspace discovery using sublime_standard_tools
impl WorkspaceManager {
    pub fn discover_workspace(&self, path: impl AsRef<Path>, options: DiscoveryOptions) 
        -> Result<Workspace, WorkspaceError> 
    {
        // Use sublime_standard_tools to find project root
        let root_path = if options.auto_detect_root {
            sublime_standard_tools::get_project_root_path(Some(PathBuf::from(path.as_ref())))
                .ok_or(WorkspaceError::RootNotFound)?
        } else {
            PathBuf::from(path.as_ref())
        };
        
        // Detect package manager
        let package_manager = if options.detect_package_manager {
            sublime_standard_tools::detect_package_manager(&root_path)
        } else {
            None
        };
        
        // Continue implementation...
    }
}

// Task execution using sublime_standard_tools
impl<'a> TaskRunner<'a> {
    fn execute_task(&self, task: &Task) -> Result<TaskResult, TaskError> {
        // Implementation using sublime_standard_tools::execute
    }
}
```

### Integration with sublime\_git\_tools

```rust
// Change detection using sublime_git_tools
impl ChangeTracker {
    pub fn detect_changes_between(
        &self, 
        from_ref: &str, 
        to_ref: Option<&str>
    ) -> Result<Vec<Change>, ChangeError> {
        let repo = self.workspace.git_repo()
            .ok_or(ChangeError::NoGitRepository)?;
        
        // Use sublime_git_tools to get changed files
        let changed_files = repo.get_all_files_changed_since_sha_with_status(from_ref)?;
        
        // Get commits for detailed information
        let commits = repo.get_commits_since(Some(from_ref.to_string()), &None)?;
        
        // Process changes into Change objects
        // ...
    }
}
```

### Integration with sublime\_package\_tools

```rust
// Using PackageInfo directly from sublime_package_tools
impl Workspace {
    pub fn build_dependency_graph(&self) -> Result<DependencyGraph<'_, Package>, WorkspaceError> {
        // Convert PackageInfo references to a vector of Package objects
        let packages: Vec<Package> = self.package_infos.iter()
            .map(|info| info.borrow().package.borrow().clone())
            .collect();
            
        // Use sublime_package_tools to build the graph
        Ok(sublime_package_tools::build_dependency_graph_from_packages(&packages))
    }
    
    pub fn validate(&self) -> Result<ValidationReport, WorkspaceError> {
        let graph = self.build_dependency_graph()?;
        
        // Use sublime_package_tools validation
        graph.validate_package_dependencies()
            .map_err(|e| WorkspaceError::DependencyResolutionError(e))
    }
}
```

## Workflow Diagrams

### 1. Change Detection and Version Management Workflow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Code Changes   │────►│  Git Commits    │────►│ Change Tracker  │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Changelogs    │◄────│ Version Manager │◄────│   Changesets    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                        │                       │
        │                        │                       │
        ▼                        ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Release Workflow                         │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Git Hooks Integration Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Developer      │────►│  Git Stage      │────►│  Pre-commit     │
│  Makes Changes  │     │  Changes        │     │     Hook        │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Changeset     │◄────│    Validate     │     │    Validate     │
│   Creation      │     │   Changesets    │◄────┤  Package.json   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │
        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Git Commit     │────►│  Post-commit    │────►│ Auto-record     │
│                 │     │     Hook        │     │    Changes      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### 3. Task Execution Workflow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Task           │────►│  Dependency     │────►│Task Prioritizer │
│  Definition     │     │  Resolution     │     │                 │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Result         │◄────│ Task Execution  │◄────│Parallel Executor│
│  Collection     │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### 4. Deployment Management Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Environment    │────►│  Deployment     │────►│   Version       │
│  Configuration  │     │    Planning     │     │   Selection     │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Deployment     │◄────│   Deployment    │◄────│   Approval      │
│     History     │     │   Execution     │     │    Process      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Advanced Considerations

### 1. Performance Optimization

* **Incremental Analysis**: Only analyze changed packages and their dependents
* **Caching**: Cache dependency graphs, parsing results, and other expensive computations
* **Parallel Processing**: Utilize multi-threading for CPU-intensive operations
* **Lazy Loading**: Defer loading package details until needed
* **Differential Operations**: Support partial updates to large workspaces

### 2. Security

* **Hook Validation**: Verify hook integrity to prevent tampering
* **Dependency Security**: Integrate with vulnerability scanning
* **Permission Management**: Fine-grained controls for various operations
* **Audit Trails**: Track all operations for accountability

### 3. Extensibility

* **Plugin System**: Support for third-party extensions
* **Custom Validators**: Allow custom validation rules
* **Event Hooks**: Provide event hooks for external tools
* **Configuration Flexibility**: Support project-specific customizations

### 4. Enterprise Features

* **Multi-Team Support**: Handle repositories with multiple teams
* **Compliance**: Support for enterprise compliance requirements
* **Integration Points**: Connect with enterprise systems (JIRA, etc.)
* **Advanced Reporting**: Generate comprehensive reports for stakeholders

### 5. Cross-Language Support

* **Polyglot Repositories**: Support for mixed language repositories
* **Language-Specific Extensions**: Accommodate language-specific requirements
* **Cross-Language Dependencies**: Track dependencies across language boundaries

### 6. Enhanced Developer Experience

* **Interactive TUI**: Terminal UI for complex operations
* **Guided Workflows**: Step-by-step guidance for common tasks
* **Rich Error Messages**: Clear, actionable error messages
* **Documentation Generation**: Auto-generate package documentation

## Implementation Strategy

### Phase 1: Core Framework (2-3 weeks)

1. **Workspace Management**
   * Package discovery and loading
   * Integration with `sublime_standard_tools` for command execution
   * Integration with `sublime_git_tools` for repo operations
   * Basic dependency graph via `sublime_package_tools`
2. **Change Management**
   * Change tracking infrastructure
   * Git integration for change detection
   * Changeset storage system
3. **Core Testing Infrastructure**
   * Unit tests for core components
   * Integration tests with sample repos

### Phase 2: Extended Functionality (2-3 weeks)

1. **Versioning System**
   * Version bumping strategies
   * Changelog generation
   * Integrated version validation
2. **Task Execution**
   * Task definition and discovery
   * Parallel execution engine
   * Task dependency resolution
3. **Git Hooks**
   * Hook manager implementation
   * Template system
   * Pre-commit and post-commit hooks

### Phase 3: Advanced Features (2-3 weeks)

1. **Deployment Management**
   * Environment configuration
   * Deployment tracking
   * Release history
2. **External Integration**
   * Turborepo config generation
   * Nx integration
   * CI system connectors
3. **Performance Optimization**
   * Caching systems
   * Incremental analysis
   * Parallel processing

### Phase 4: Node.js Bindings and Documentation (1-2 weeks)

1. **NAPI Interface**
   * Core binding infrastructure
   * JavaScript/TypeScript typings
   * Error handling translation
2. **Documentation**
   * API documentation
   * User guides
   * Examples and tutorials
3. **Final Testing**
   * End-to-end tests
   * Performance benchmarks
   * Cross-platform validation

## Examples and Use Cases

### Example: Setting Up a Release Workflow

```rust
// Create workspace
let workspace_manager = WorkspaceManager::new();
let workspace = workspace_manager.discover_workspace("./", DiscoveryOptions::new())?;

// Setup change tracking
let store = FileChangeStore::new("./.changeset");
let change_tracker = ChangeTracker::new(Rc::new(workspace.clone()), Box::new(store));

// Create version manager
let version_manager = VersionManager::new(&workspace, Some(&change_tracker));

// Generate and preview version bumps
let strategy = VersionBumpStrategy::ConventionalCommits { from_ref: Some("v1.0.0".to_string()) };
let preview = version_manager.preview_bumps(&strategy)?;

println!("Planned version changes:");
for change in &preview.changes {
    println!("{}: {} -> {}", change.package, change.from, change.to);
}

// Apply version bumps if confirmed
let changes = version_manager.apply_bumps(&strategy, false)?;

// Generate changelogs
let changelog_options = ChangelogOptions::new();
let changelogs = version_manager.generate_changelogs(&changelog_options, false)?;

// Run tests before publishing
let task_runner = TaskRunner::new(&workspace, None);
task_runner.add_task(Task::new("test", "npm test"));
let results = task_runner.run_all()?;

// Check if all tests passed
let all_passed = results.iter().all(|r| r.success);
if all_passed {
    println!("All tests passed, ready to publish!");
} else {
    println!("Some tests failed, aborting release.");
}
```

### Example: Git Hook Setup

```rust
// Create workspace and hooks manager
let workspace_manager = WorkspaceManager::new();
let workspace = workspace_manager.discover_workspace("./", DiscoveryOptions::new())?;
let hooks_manager = HooksManager::new(&workspace);

// Install Git hooks
let options = HookInstallOptions {
    overwrite_existing: true,
    backup_existing: true,
    env_vars: HashMap::new(),
    custom_templates: HashMap::new(),
};

hooks_manager.install_hooks(
    &[HookType::PreCommit, HookType::PostCommit],
    &options
)?;

println!("Git hooks installed successfully!");

// Check if hooks are working
let result = hooks_manager.run_hook(HookType::PreCommit)?;
if result.success {
    println!("Pre-commit hook validated successfully");
} else {
    println!("Pre-commit hook validation failed: {:?}", result.errors);
}
```

## Conclusion

`sublime_monorepo_tools` provides a comprehensive solution for JavaScript/TypeScript monorepo management, building on the strengths of the existing Workspace Node Tools ecosystem. By integrating with `sublime_standard_tools`, `sublime_git_tools`, and `sublime_package_tools`, it delivers a cohesive and powerful system for managing complex monorepos.

The Git hooks integration ensures a seamless developer workflow, while the extensive versioning, deployment, and task management capabilities enable efficient operation of large-scale monorepos. Both Rust and Node.js interfaces make the system accessible to a wide range of users and use cases, from command-line tools to full-featured desktop applications.

This comprehensive system design provides a solid foundation for implementing `sublime_monorepo_tools` as a powerful addition to the Workspace Node Tools ecosystem.
