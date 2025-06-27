# Monorepo Tools Architecture - Ownership Boundaries

This document defines clear ownership boundaries for all components in the monorepo tools crate, establishing who owns what data and how components interact without violating Rust's ownership principles.

## Core Principles

1. **Single Owner**: Each piece of data has exactly one owner at any given time
2. **Move Semantics**: Components take ownership via move and return it via `into_` methods
3. **No Shared State**: No Arc<RwLock<>>, no Rc<RefCell<>>, no global state
4. **Explicit Interfaces**: All inter-component communication through well-defined interfaces

## Component Ownership Map

### 1. MonorepoProject (Core Aggregate)

**Owns:**
- `sublime_standard_tools::monorepo::Project` - Base project structure
- `Vec<MonorepoPackageInfo>` - All package information
- `ConfigManager` - Configuration management
- `DependencyRegistry` - Shared dependency instances
- `VersionManager` - Version management strategies
- `MonorepoTools` - Tool integrations

**Ownership Rules:**
- Created once at application start
- Passed by reference (`&self` or `&mut self`) to most operations
- Never cloned or wrapped in Arc

### 2. ConfigManager

**Owns:**
- `MonorepoConfig` - The actual configuration data
- `Option<PathBuf>` - Configuration file path
- `bool` - Auto-save flag

**Ownership Transfer:**
- Takes ownership of config during creation
- Methods like `with_update(self) -> Self` consume and return ownership
- Facade over focused components that follow the same pattern

**Sub-components:**
- `ConfigPersistence`: Handles file I/O operations
- `ConfigReader`: Provides read-only access
- `ConfigWriter`: Handles updates
- `WorkspacePatternManager`: Manages workspace patterns
- `PatternMatcher`: Pattern matching utilities

### 3. MonorepoPackageInfo

**Owns:**
- `PackageInfo` (from package-tools) - Package.json data
- `WorkspacePackage` (from standard-tools) - Workspace metadata
- `Vec<Changeset>` - Package changesets
- `Vec<String>` - Dependents and external dependencies
- `VersionStatus` - Current version status

**Ownership Transfer:**
- Components take ownership via `new(package: MonorepoPackageInfo)`
- Return ownership via `into_package(self) -> MonorepoPackageInfo`
- Facade pattern maintains backward compatibility

**Sub-components:**
- `PackageInfoReader`: Read-only access (borrows `&MonorepoPackageInfo`)
- `PackageVersionManager`: Version operations (takes ownership)
- `PackageChangesetManager`: Changeset operations (takes ownership)
- `PackageDependencyManager`: Dependency management (takes ownership)
- `PackagePersistence`: File operations (takes ownership)

### 4. Event System

**Owns:**
- `EventBus`: Central event dispatcher
  - `RwLock<HashMap<Uuid, EventSubscription>>` - Subscriptions
  - `broadcast::Sender<MonorepoEvent>` - Event channel
  - `RwLock<BinaryHeap<QueuedEvent>>` - Priority queue
  - `RwLock<EventBusStats>` - Statistics

**Ownership Rules:**
- EventBus is created once and lives for the application lifetime
- Events are cloned when broadcast (they contain only data, no handles)
- Handlers receive events by value

### 5. Hook System

**Owns:**
- `HookManager`:
  - `HashMap<HookType, HookDefinition>` - Default hooks
  - `HashMap<HookType, HookDefinition>` - Custom hooks
  - `HashMap<String, Arc<dyn HookExecutor>>` - Executors
  - `Option<EventBus>` - Event emitter

**Ownership Rules:**
- Hook definitions are owned, not shared
- Executors use Arc only for trait objects (necessary for async)
- No global state or lazy initialization

### 6. Task System

**Owns:**
- `TaskManager`:
  - `HashMap<String, TaskDefinition>` - Task definitions
  - `TaskDependencyGraph` - Dependency graph
  - Task execution state

**Ownership Rules:**
- Tasks are defined once and owned by the manager
- Execution creates new owned state each time
- Results are moved out to the caller

### 7. Analysis System

**Owns:**
- `AnalysisEngine`:
  - Analysis configuration
  - Temporary analysis state during execution

**Ownership Rules:**
- Takes references to project data
- Creates and owns analysis results
- Returns owned results to caller

## Inter-Component Communication

### 1. Direct Method Calls
- Components expose public methods that take references
- No circular dependencies between components
- Clear caller/callee relationships

### 2. Event-Driven Communication
- Components emit events for state changes
- Other components subscribe to relevant events
- Loose coupling, no direct dependencies

### 3. Dependency Injection Interfaces
- Traits define required capabilities
- Components depend on traits, not concrete types
- Enables testing and modularity

## Data Flow Patterns

### 1. Read Operations
```rust
// Borrow reference for reading
let reader = PackageInfoReader::new(&package);
let name = reader.name();
```

### 2. Update Operations
```rust
// Take ownership, modify, return ownership
let manager = PackageVersionManager::new(package);
let updated_package = manager
    .update_version("2.0.0")?
    .into_package();
```

### 3. Batch Operations
```rust
// Chain operations using ownership transfer
let package = PackageVersionManager::new(package)
    .bump_version(VersionBumpType::Minor)?
    .into_package();

let package = PackageChangesetManager::new(package)
    .add_changeset(changeset)
    .into_package();
```

## Lifetime Management

### 1. Application Lifetime
- `MonorepoProject` - Lives for entire application
- `EventBus` - Lives for entire application
- Base crate instances (git, file system) - Application lifetime

### 2. Operation Lifetime
- Component managers - Created per operation
- Analysis results - Owned by caller after operation
- Events - Short-lived, processed immediately

### 3. No Shared Lifetime
- No Rc/Arc for business objects
- No RefCell for interior mutability
- Clear ownership at all times

## Migration Guidelines

When adding new components:

1. **Define Ownership**: Clearly state what data the component owns
2. **Use Move Semantics**: Take ownership via `new()`, return via `into_*()`
3. **Avoid Sharing**: Don't use Arc/Rc unless absolutely necessary
4. **Document Boundaries**: Update this document with new components
5. **Test Ownership**: Write tests that verify ownership transfers

## Benefits

1. **No Runtime Overhead**: No Arc/Mutex overhead
2. **Compile-Time Safety**: Rust enforces ownership rules
3. **Clear Mental Model**: Easy to reason about data flow
4. **No Deadlocks**: No locks, no deadlock possibility
5. **Predictable Performance**: No hidden synchronization costs