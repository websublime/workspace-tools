---
type: Page
collections: Workspace Project Docs
title: 'Monorepo Management System: Comprehensive Implementation Plan'
description: null
icon: 💭
createdAt: '2025-04-14T23:57:55.769Z'
creationDate: 2025-04-15 00:57
modificationDate: 2025-04-15 01:13
tags: [workspace-tools, rust, cli]
coverImage: null
---

### The request

let's create detail plan and a step by step implementation. Diagrams to demonstrate flow, communication etc and do not forget all the initial api about the project, many things can be used from there, and as to be because all those sublime_tools_... apis are the core of it. So points of consideration:

- sublime_tools_... crates api

- a base command

- all the others are sub commands (cargo style) and ability to extend from outside

- monitor all the changes in the monorepo

- daemon as to manage and deal with multiple repos

- everything that  it can be configured it should be exposed from a config (toml)

- a ratatui awesome app (this we will need to go deep to know how it should be)

    - shows current branch

    - show curren folder of the monorepo

    - show packages that exist on the monorepo (name, version etc. this can be getted from the sublime_tools_... api)

    - ability to analyse dependencies

    - ability to upgrade packages (maybe from time to time ask registry about those external packages version)

    - ability to connect to github and receive updates from there (like action status, users commits etc)

    - files changed

    - package bumps can be done (suggest, preview)

    - changesets visualization

    - graph visualization

    - other things that the currents apis from sublime_tools_... crates offer

To finalize, for now no code just a very well detailed plan like describe here, an api spec definition for to you have in the next sessions the know how of what we are doing and with that defined i will pick a phase/point and ask you to work on that only, so we can build the solution step by step.

---

## System Architecture Overview

```text
┌───────────────────────────────────────────────────────────────────────────┐
│                                                                           │
│               ┌────────────────┐                                          │
│               │  workspace     │  Main CLI entrypoint                     │
│               │  (Base Command)│  - Subcommand routing                    │
│               └───────┬────────┘  - Help/version info                     │
│                       │                                                   │
│                       ▼                                                   │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │                         Subcommand Handlers                          │ │
│  │                                                                      │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────────┐        │ │
│  │  │  daemon    │ │  monitor   │ │  changes   │ │  version    │  ...   │ │
│  │  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘ └──────┬──────┘        │ │
│  └────────┼───────────────┼───────────────┼───────────────┼─────────────┘ │
│           │               │               │               │               │
│           ▼               │               │               │               │
│  ┌────────────────┐       │               │               │               │
│  │                │       │               │               │               │
│  │  Daemon Service│◄──────┘               │               │               │
│  │                │◄──────────────────────┘               │               │
│  └────────┬───────┘◄──────────────────────────────────────┘               │
│           │                                                               │
│           ▼                                                               │
│  ┌────────────────────────────────────────┐                               │
│  │          Repository Management         │                               │
│  │                                        │                               │
│  │  ┌─────────────┐  ┌─────────────────┐  │                               │
│  │  │ Repository 1│  │  Repository 2   │  │                               │
│  │  │ Watchers    │  │  Watchers       │  │                               │
│  │  └─────────────┘  └─────────────────┘  │                               │
│  └────────────────────────────────────────┘                               │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

## Integration with Sublime Tools Crates

```text
┌───────────────────────────────────────────────────────────────────────────┐
│                         workspace-cli                                     │
│                                                                           │
│  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐             │
│  │  CLI Interface │   │  Daemon Service│   │  TUI Interface │             │
│  └───────┬────────┘   └───────┬────────┘   └───────┬────────┘             │
│          │                    │                    │                      │
│          │                    │                    │                      │
│  ┌───────▼────────────────────▼────────────────────▼───────────────────┐  │
│  │                      Integration Layer                              │  │
│  └───────┬────────────────────┬────────────────────┬───────────────────┘  │
│          │                    │                    │                      │
│          ▼                    ▼                    ▼                      │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐               │
│  │sublime_git_  │     │sublime_      │     │sublime_      │               │
│  │tools         │     │package_tools │     │monorepo_tools│               │
│  └──────┬───────┘     └──────┬───────┘     └──────┬───────┘               │
│         │                    │                    │                       │
│         └────────────────────┼────────────────────┘                       │
│                              ▼                                            │
│                     ┌──────────────┐                                      │
│                     │sublime_      │                                      │
│                     │standard_tools│                                      │
│                     └──────────────┘                                      │
└───────────────────────────────────────────────────────────────────────────┘
```

## Communication Flow Diagram

```text
┌────────────┐          ┌────────────┐          ┌────────────┐
│            │          │            │          │            │
│  CLI       │◄─IPC────►│  Daemon    │◄─IPC────►│  Monitor   │
│  Commands  │          │  Service   │          │  (TUI)     │
│            │          │            │          │            │
└────────────┘          └─────┬──────┘          └────────────┘
                              │
                              │ Notifications
                              ▼
                        ┌────────────┐          ┌────────────┐
                        │            │          │            │
                        │  Event     │◄─HTTP───►│  GitHub    │
                        │  System    │          │  API       │
                        │            │          │            │
                        └─────┬──────┘          └────────────┘
                              │
                              │ Polling
                              ▼
                        ┌────────────┐
                        │ Repository │
                        │ Watchers   │
                        │            │
                        └────────────┘
```

## Component Specifications

### 1. Base Command (`workspace`)

**Purpose**: Central entry point that routes to subcommands

**Responsibilities**:

- Parse initial command-line arguments

- Route to appropriate subcommand

- Provide help and version information

- Handle subcommand discovery (built-in and external)

**Integration Points**:

- Calls to subcommand binaries

- Configuration reading

### 2. Daemon Subcommand (`workspace daemon`)

**Purpose**: Manage the background service for repository monitoring

**Responsibilities**:

- Start/stop/restart the daemon

- Configure daemon behavior

- Report daemon status

**Integration Points**:

- Uses `sublime_standard_tools` for process management

- Uses `sublime_git_tools` for Git operations

- Uses IPC for communication with monitor

### 3. Monitor Subcommand (`workspace monitor`)

**Purpose**: Interactive TUI for real-time repository monitoring

**Responsibilities**:

- Display repository status

- Show file changes

- Visualize package information

- Provide interactive workspace management

**Integration Points**:

- Uses `sublime_monorepo_tools` for workspace analysis

- Uses `sublime_package_tools` for dependency information

- Uses IPC to communicate with daemon

### 4. Configuration Management

**Purpose**: Handle user configuration across components

**Responsibilities**:

- Read/write TOML configuration files

- Provide defaults

- Validate configuration values

**Integration Points**:

- File system for configuration storage

- All components for configuration consumption

### 5. Repository Registry

**Purpose**: Track multiple repositories for monitoring

**Responsibilities**:

- Register/unregister repositories

- Store repository metadata

- Configure per-repository settings

**Integration Points**:

- Daemon for active monitoring

- Configuration system for persistence

### 6. File System Watcher

**Purpose**: Detect file changes in repositories

**Responsibilities**:

- Watch for file system events

- Filter events based on configuration

- Report changes to daemon

**Integration Points**:

- `notify` crate for file system events

- Daemon service for event processing

### 7. TUI Interface

**Purpose**: Provide rich interactive monitoring experience

**Responsibilities**:

- Render workspace information

- Accept user input

- Provide visualizations (graphs, tables)

- Support workspace operations

**Integration Points**:

- `ratatui` for terminal UI

- `sublime_monorepo_tools` for workspace data

- IPC for daemon communication

### 8. GitHub Integration

**Purpose**: Fetch and display GitHub repository information

**Responsibilities**:

- Authenticate with GitHub

- Fetch PR, issue, and CI status

- Report events to the daemon

**Integration Points**:

- GitHub API

- Event system for notifications

## Data Flow Specifications

### 1. Workspace Command to Subcommand Flow

```text
User input → workspace parse args → find subcommand → exec subcommand binary → subcommand execution
```

### 2. Daemon Communication Flow

```text
Client → Connect to socket → Send command → Daemon processes → Response returned → Client handles response
```

### 3. File Change Detection Flow

```text
FS event → Watcher receives → Filtered by patterns → Mapped to change type → Added to event queue → Processed by daemon → Sent to subscribers
```

### 4. Monitor Refresh Flow

```text
Timer event → Request updates from daemon → Receive state → Process for display → Render TUI → User interaction
```

## Configuration Schema

```text
# ~/.config/workspace-cli/config.toml
[general]
log_level = "info"
auto_start_daemon = true
[daemon]
socket_path = "~/.local/share/workspace-cli/daemon.sock" 
pid_file = "~/.local/share/workspace-cli/daemon.pid"
polling_interval_ms = 500
inactive_polling_ms = 5000
[monitor]
refresh_rate_ms = 1000
default_view = "overview"  # overview, changes, packages, graph
color_theme = "default"    # default, dark, light
[watcher]
include_patterns = ["**/*.rs", "**/*.toml", "**/*.js", "**/*.ts"]
exclude_patterns = ["**/node_modules/**", "**/target/**", "**/.git/**"]
use_git_hooks = true
[github]
enable_integration = false
# Token can be set via environment variable WORKSPACE_GITHUB_TOKEN
token_path = "~/.config/workspace-cli/github_token"
fetch_interval_s = 300
[[repositories]]
path = "/path/to/repo1"
name = "project1"
active = true
branch = "main"
include_patterns = ["packages/**/*.rs"]  # Override global patterns
```

## Repository Registry Schema

```text
# ~/.local/share/workspace-cli/registry.toml
[[repositories]]
path = "/path/to/repo1"
name = "project1"
active = true
last_activity = "2023-06-01T12:34:56Z"
package_manager = "cargo"
include_patterns = []
exclude_patterns = []
[[repositories]]
path = "/path/to/repo2"
name = "project2"
active = false
last_activity = "2023-05-31T10:11:12Z"
package_manager = "npm"
include_patterns = []
exclude_patterns = []
```

## API Integration Specification

### Integration with `sublime_git_tools`

```rust
// Key functions from sublime_git_tools we'll use:
// Repository operations
Repo::open(path) -> Result<Repo, RepoError>
repo.get_current_branch() -> Result<String, RepoError>
repo.get_current_sha() -> Result<String, RepoError>
// Change detection
repo.status_porcelain() -> Result<Vec<String>, RepoError>
repo.get_all_files_changed_since_sha_with_status(git_ref) -> Result<Vec<GitChangedFile>, RepoError>
// Commit operations
repo.get_commits_since(since, relative) -> Result<Vec<RepoCommit>, RepoError>
```

### Integration with `sublime_package_tools`

```rust
// Key functions from sublime_package_tools we'll use:
// Package operations
PackageInfo::new(package, path, json) -> Self
package_info.update_version(new_version) -> Result<(), VersionError>
// Dependency graph
build_dependency_graph_from_packages(packages) -> DependencyGraph<'_, Package>
graph.detect_circular_dependencies() -> &Self
graph.validate_package_dependencies() -> Result<ValidationReport, DependencyResolutionError>
// Visualization
generate_dot(graph, options) -> Result<String, std::fmt::Error>
generate_ascii(graph) -> Result<String, std::fmt::Error>
```

### Integration with `sublime_monorepo_tools`

```rust
// Key functions from sublime_monorepo_tools we'll use:
// Workspace operations
WorkspaceManager::new() -> Self
manager.discover_workspace(path, options) -> Result<Workspace, WorkspaceError>
workspace.get_package(name) -> Option<Rc<RefCell<PackageInfo>>>
workspace.sorted_packages() -> Vec<Rc<RefCell<PackageInfo>>>
workspace.affected_packages(changed_packages) -> Vec<Rc<RefCell<PackageInfo>>>
// Change tracking
ChangeTracker::new(workspace, store) -> Self
tracker.detect_changes_between(from_ref, to_ref) -> ChangeResult<Vec<Change>>
tracker.unreleased_changes() -> ChangeResult<HashMap<String, Vec<Change>>>
// Version management
VersionManager::new(workspace, change_tracker) -> Self
manager.preview_bumps(strategy) -> VersioningResult<VersionBumpPreview>
manager.apply_bumps(strategy, dry_run) -> VersioningResult<Vec<PackageVersionChange>>
```

### Integration with `sublime_standard_tools`

```rust
// Key functions from sublime_standard_tools we'll use:
// Command execution
execute(cmd, path, args, process) -> CommandResult<T>
// Package manager detection
detect_package_manager(path) -> Option<CorePackageManager>
// Project path resolution
get_project_root_path(root) -> Option<PathBuf>
```

## TUI Screen Designs

### 1. Overview Screen

```text
┌─────────────────────────────────────────────────────────────────┐
│ Workspace Monitor                           [q]uit [r]efresh    │
├─────────────────────────────────────────────────────────────────┤
│ Repository: /home/user/projects/my-monorepo                     │
│ Branch: main (3 ahead, 0 behind origin/main)                    │
│ Package Manager: npm                                            │
├─────────────────────────────────────────────────────────────────┤
│ Packages [10 total]                                             │
│                                                                 │
│  Name         | Version   | Status      | Dependencies          │
│  core         | 1.0.0     | ✓ Up to date| 5 internal, 3 external│
│  ui-components| 0.8.1     | ✓ Up to date| 2 internal, 8 external│
│  api-client   | 1.2.3     | ! Modified  | 1 internal, 2 external│
│  cli          | 0.9.5     | ! Modified  | 3 internal, 4 external│
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Recent Changes                                                  │
│                                                                 │
│  File                          | Status   | Package             │
│  packages/api-client/src/lib.rs| Modified | api-client          │
│  packages/cli/src/commands.rs  | Modified | cli                 │
│  packages/cli/package.json     | Modified | cli                 │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Recent Commits                                                  │
│                                                                 │
│  d8f3a2e feat: add new API endpoint (John Doe, 10m ago)         │
│  c7e5b1d fix: resolve dependency issue (Jane Smith, 1h ago)     │
│  a1b2c3d chore: update dependencies (John Doe, 3h ago)          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Changes Screen

```text
┌─────────────────────────────────────────────────────────────────┐
│ Workspace Monitor > Changes                  [q]uit [r]efresh   │
├─────────────────────────────────────────────────────────────────┤
│ Filter: [x] All [ ] Modified [ ] Added [ ] Deleted              │
├─────────────────────────────────────────────────────────────────┤
│ Changed Files                                                   │
│                                                                 │
│  File                          | Status   | Package    | Lines  │
│  packages/api-client/src/lib.rs| Modified | api-client | +15 -5 │
│  packages/cli/src/commands.rs  | Modified | cli        | +28 -10│
│  packages/cli/package.json     | Modified | cli        | +3 -1  │
│  docs/API.md                   | Added    | -          | +120 -0│
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Affected Packages                                               │
│                                                                 │
│  Package       | Direct Changes | Affected By | Status          │
│  api-client    | Yes            | -           | Modified        │
│  cli           | Yes            | -           | Modified        │
│  core          | No             | cli         | Affected        │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Diff View (packages/api-client/src/lib.rs)                      │
│                                                                 │
│  @@ -45,10 +45,20 @@                                            │
│   pub fn create_client(config: &Config) -> Result<Client, Error> {│
│  -    let timeout = config.timeout.unwrap_or(30);               │
│  +    let timeout = config.timeout.unwrap_or(60);               │
│       let client = Client::new()                                │
│           .timeout(Duration::from_secs(timeout))                │
│  +        .user_agent("API Client/1.2.3")                       │
│           .build()?;                                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Dependency Graph Screen

```text
┌─────────────────────────────────────────────────────────────────┐
│ Workspace Monitor > Dependency Graph          [q]uit [r]efresh  │
├─────────────────────────────────────────────────────────────────┤
│ View: [x] Internal [ ] External [ ] All                         │
├─────────────────────────────────────────────────────────────────┤
│                                  ┌─────────┐                     │
│                     ┌───────────►│  core   │◄───────┐            │
│                     │            └────┬────┘        │            │
│                     │                 │             │            │
│           ┌─────────┴─┐              │           ┌─┴──────────┐  │
│           │  cli      │◄─────────────┘           │api-client  │  │
│           └───────────┘                          └────────────┘  │
│                ▲                                       ▲         │
│                │                                       │         │
│          ┌─────┴─────┐                          ┌─────┴─────┐    │
│          │ui-components                         │server     │    │
│          └───────────┘                          └───────────┘    │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Dependencies Analysis                                           │
│                                                                 │
│  Circular Dependencies: None detected                           │
│  External Dependencies: 24 total                                │
│  Version Conflicts:                                             │
│    - lodash: 4.17.20 (ui-components), 4.17.21 (api-client)      │
│    - react: 17.0.2 (ui-components), 18.0.0 (cli)                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Version Management Screen

```text
┌─────────────────────────────────────────────────────────────────┐
│ Workspace Monitor > Version Management        [q]uit [r]efresh  │
├─────────────────────────────────────────────────────────────────┤
│ Strategy: [x] Independent [ ] Synchronized [ ] Conventional     │
├─────────────────────────────────────────────────────────────────┤
│ Version Suggestions                                             │
│                                                                 │
│  Package       | Current | Suggested | Type  | Reason           │
│  api-client    | 1.2.3   | 1.3.0     | Minor | New features     │
│  cli           | 0.9.5   | 0.9.6     | Patch | Bug fixes        │
│  core          | 1.0.0   | 1.0.0     | None  | No changes       │
│  ui-components | 0.8.1   | 0.8.1     | None  | No changes       │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Unreleased Changes                                              │
│                                                                 │
│  Package    | Change Type | Description                         │
│  api-client | Feature     | Add timeout configuration           │
│  api-client | Feature     | Add user agent string               │
│  cli        | Fix         | Resolve command parsing issue       │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Actions: [b]ump versions [p]review changelog [c]reate changeset │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Foundation

#### Step 1: Project Setup

1. Create project structure with Cargo workspace

2. Set up multi-binary targets

3. Configure dependencies

4. Create basic CLI entry point

#### Step 2: Configuration System

1. Implement TOML configuration reading/writing

2. Add default configuration values

3. Setup configuration discovery

#### Step 3: Base Command

1. Implement main entry point

2. Add subcommand routing

3. Implement help and version commands

4. Setup external subcommand discovery

### Phase 2: Core Components

#### Step 4: Daemon Implementation

1. Create daemon service structure

2. Implement process management (start/stop/status)

3. Add IPC server for communication

4. Implement basic daemon commands

#### Step 5: Repository Management

1. Create repository registry

2. Implement repository registration/unregistration

3. Add repository metadata storage

4. Integrate with configuration system

#### Step 6: File System Monitoring

1. Implement file system watcher

2. Add file event filtering

3. Create change detection logic

4. Add integration with `sublime_git_tools`

### Phase 3: User Interface

#### Step 7: Monitor TUI Foundation

1. Set up Ratatui framework

2. Implement basic layout and navigation

3. Create standard components (tables, tabs, status bars)

4. Add key input handling

#### Step 8: Monitor Views

1. Implement overview screen

2. Add changes screen

3. Create dependency graph visualization

4. Implement version management interface

#### Step 9: Data Integration

1. Connect TUI with daemon via IPC

2. Implement data refreshing logic

3. Add real-time updates

4. Create data caching for performance

### Phase 4: Integration & Advanced Features

#### Step 10: Workspace Integration

1. Integrate with `sublime_monorepo_tools` for workspace operations

2. Add package discovery and analysis

3. Implement dependency graph building

4. Add workspace validation

#### Step 11: Change Tracking

1. Implement change detection using Git

2. Add changeset management

3. Integrate with version bump suggestions

4. Implement changelog generation

#### Step 12: GitHub Integration

1. Add GitHub authentication

2. Implement PR and issue fetching

3. Add CI status monitoring

4. Create event notifications for GitHub events

### Phase 5: Polish & Deployment

#### Step 13: Performance Optimization

1. Optimize file watching for large repositories

2. Add incremental updates

3. Implement lazy loading for TUI views

4. Add caching for frequently used data

#### Step 14: Error Handling & Logging

1. Implement comprehensive error handling

2. Add structured logging

3. Create diagnostics reporting

4. Implement user-friendly error messages

#### Step 15: Documentation & Testing

1. Create user documentation

2. Add integration tests

3. Implement E2E testing

4. Create installation and usage guides

## Detailed Component Specs

### Daemon Service API

```rust
// Core daemon functionality
struct DaemonService {
    config: Config,
    repositories: HashMap<String, Repository>,
    event_queue: VecDeque<DaemonEvent>,
    ipc_server: IpcServer,
}
impl DaemonService {
    // Create a new daemon service
    fn new(config: Config) -> Result<Self, DaemonError>;
    
    // Start the daemon process
    fn start() -> Result<(), DaemonError>;
    
    // Stop the daemon process
    fn stop() -> Result<(), DaemonError>;
    
    // Check if daemon is running
    fn is_running() -> Result<bool, DaemonError>;
    
    // Get daemon status information
    fn status() -> Result<DaemonStatus, DaemonError>;
    
    // Add a repository for monitoring
    fn add_repository(&mut self, path: &Path) -> Result<(), DaemonError>;
    
    // Remove a repository from monitoring
    fn remove_repository(&mut self, id: &str) -> Result<(), DaemonError>;
    
    // Main event loop
    fn run(&mut self) -> Result<(), DaemonError>;
    
    // Process pending events
    fn process_events(&mut self) -> Result<(), DaemonError>;
    
    // Handle a client connection
    fn handle_client_connection(&mut self, conn: ClientConnection);
    
    // Send notification to connected clients
    fn notify_clients(&self, event: DaemonEvent);
}
```

### IPC Protocol

```text
# Request formats (client to daemon)
STATUS                        # Get daemon status
ADD_REPO <path>               # Add repository for monitoring
REMOVE_REPO <id>              # Remove repository
LIST_REPOS                    # List all monitored repositories
GET_CHANGES <repo_id>         # Get recent changes for repository
GET_PACKAGES <repo_id>        # Get package information
ANALYZE_DEPS <repo_id>        # Analyze dependencies
SUGGEST_VERSIONS <repo_id>    # Get version suggestions
# Response formats (daemon to client)
OK <data>                      # Success with optional data
ERROR <message>                # Error with message
EVENT <type> <data>            # Event notification
```

### Monitor API

```rust
// Core monitor functionality
struct Monitor {
    client: DaemonClient,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: MonitorState,
    active_view: View,
    views: HashMap<ViewType, Box<dyn ViewRenderer>>,
}
impl Monitor {
    // Create a new monitor
    fn new() -> Result<Self, MonitorError>;
    
    // Connect to daemon
    fn connect_to_daemon() -> Result<DaemonClient, MonitorError>;
    
    // Main event loop
    fn run(&mut self) -> Result<(), MonitorError>;
    
    // Handle user input
    fn handle_input(&mut self, event: Event) -> Result<bool, MonitorError>;
    
    // Refresh data from daemon
    fn refresh_data(&mut self) -> Result<(), MonitorError>;
    
    // Switch to a different view
    fn switch_view(&mut self, view: ViewType);
    
    // Render the current view
    fn render(&mut self) -> Result<(), MonitorError>;
}
```

### Configuration API

```rust
// Configuration management
struct Config {
    general: GeneralConfig,
    daemon: DaemonConfig,
    monitor: MonitorConfig,
    watcher: WatcherConfig,
    github: GithubConfig,
    repositories: Vec<RepositoryConfig>,
}
impl Config {
    // Load configuration from default locations
    fn load() -> Result<Self, ConfigError>;
    
    // Load configuration from specific path
    fn load_from(path: &Path) -> Result<Self, ConfigError>;
    
    // Save configuration to default location
    fn save(&self) -> Result<(), ConfigError>;
    
    // Save configuration to specific path
    fn save_to(&self, path: &Path) -> Result<(), ConfigError>;
    
    // Get default configuration
    fn default() -> Self;
    
    // Add repository to configuration
    fn add_repository(&mut self, repo: RepositoryConfig);
    
    // Remove repository from configuration
    fn remove_repository(&mut self, id: &str) -> bool;
}
```

## Test Plan

### Unit Tests

1. Configuration parsing/writing

2. Command-line argument parsing

3. IPC protocol message handling

4. File system event filtering

5. Repository registry operations

### Integration Tests

1. Daemon lifecycle management

2. Monitor-to-daemon communication

3. Git repository change detection

4. Workspace package discovery

5. Version bump suggestions

### End-to-End Tests

1. Complete CLI workflow for adding repositories

2. TUI navigation and interaction

3. Change detection and visualization

4. Version bumping workflow

5. GitHub integration

## Next Steps

With this detailed plan in place, we can proceed with implementation by tackling each phase in order. The most logical first steps would be:

1. Set up the project structure

2. Implement the base command routing

3. Create the configuration system

4. Build the daemon foundation

From there, we can incrementally add features while maintaining the ability to test and use the system at each step.



---

# Enhanced Folder Structure for the Monorepo

Let's define a folder structure that integrates our CLI tool smoothly into your existing monorepo, following the established naming conventions and architecture:

```text
workspace-node-tools/
├── Cargo.toml                    # Workspace-level Cargo.toml
├── sublime_standard_tools/       # Existing crate
├── sublime_git_tools/            # Existing crate
├── sublime_package_tools/        # Existing crate
├── sublime_monorepo_tools/       # Existing crate
└── sublime_workspace_cli/        # New CLI crate
    ├── Cargo.toml                # CLI crate manifest
    ├── src/
    │   ├── main.rs               # Main CLI entry point (workspace command)
    │   ├── bin/                  # Subcommand binaries
    │   │   ├── workspace-daemon.rs     # Daemon implementation
    │   │   ├── workspace-monitor.rs    # Monitor TUI implementation
    │   │   ├── workspace-changes.rs    # Changes subcommand
    │   │   ├── workspace-version.rs    # Version management subcommand
    │   │   └── workspace-graph.rs      # Dependency graph visualization
    │   ├── common/               # Shared code across binaries
    │   │   ├── mod.rs            # Common module exports
    │   │   ├── config.rs         # Configuration management
    │   │   ├── paths.rs          # Path utilities
    │   │   ├── errors.rs         # Error types and handling
    │   │   └── ipc.rs            # Inter-process communication
    │   ├── daemon/               # Daemon implementation
    │   │   ├── mod.rs            # Daemon module exports
    │   │   ├── service.rs        # Daemon service implementation
    │   │   ├── watcher.rs        # File system watcher
    │   │   ├── registry.rs       # Repository registry
    │   │   └── server.rs         # IPC server implementation
    │   ├── monitor/              # Monitor implementation
    │   │   ├── mod.rs            # Monitor module exports
    │   │   ├── app.rs            # Main TUI application
    │   │   ├── state.rs          # Application state management
    │   │   ├── client.rs         # Daemon client for monitor
    │   │   └── views/            # TUI views
    │   │       ├── mod.rs        # View registry
    │   │       ├── overview.rs   # Overview screen
    │   │       ├── changes.rs    # Changes screen
    │   │       ├── graph.rs      # Dependency graph screen
    │   │       └── version.rs    # Version management screen
    │   ├── ui/                   # UI components
    │   │   ├── mod.rs            # UI module exports
    │   │   ├── components.rs     # Reusable UI components
    │   │   ├── styles.rs         # UI styling and themes
    │   │   └── widgets/          # Custom widgets
    │   │       ├── mod.rs        # Widget registry
    │   │       ├── dependency_graph.rs  # Graph visualization widget
    │   │       ├── diff_view.rs  # Code diff widget
    │   │       └── status_bar.rs # Status bar widget
    │   ├── integration/          # Integration with other tools
    │   │   ├── mod.rs            # Integration module exports
    │   │   ├── git.rs            # Git integration
    │   │   ├── github.rs         # GitHub API integration
    │   │   └── ci.rs             # CI system integration
    │   └── lib.rs                # Library exports for inter-binary sharing
    ├── assets/                   # Static assets
    │   ├── default_config.toml   # Default configuration template
    │   └── themes/               # UI themes
    │       ├── default.toml      # Default theme
    │       └── dark.toml         # Dark theme
    ├── examples/                 # Example usages and configs
    │   ├── config.toml           # Example configuration
    │   └── scripts/              # Example integration scripts
    └── tests/                    # Integration tests
        ├── cli_tests.rs          # CLI interface tests
        ├── daemon_tests.rs       # Daemon functionality tests
        └── monitor_tests.rs      # Monitor UI tests
```

## Root `Cargo.toml` Updates

To integrate the new CLI crate into the monorepo's workspace, the root `Cargo.toml` would need to be updated:

```text
[workspace]
members = [
    "sublime_standard_tools",
    "sublime_git_tools",
    "sublime_package_tools", 
    "sublime_monorepo_tools",
    "sublime_workspace_cli"
]
[workspace.dependencies]
sublime_standard_tools = { path = "sublime_standard_tools" }
sublime_git_tools = { path = "sublime_git_tools" }
sublime_package_tools = { path = "sublime_package_tools" }
sublime_monorepo_tools = { path = "sublime_monorepo_tools" }
```

## CLI Crate `Cargo.toml`

```text
[package]
name = "sublime_workspace_cli"
version = "0.1.0"
edition = "2021"
description = "CLI tools for monorepo management"
authors = ["Your Name <your.email@example.com>"]
license = "MIT OR Apache-2.0"
[dependencies]
sublime_standard_tools = { workspace = true }
sublime_git_tools = { workspace = true }
sublime_package_tools = { workspace = true }
sublime_monorepo_tools = { workspace = true }
# CLI dependencies
clap = { version = "4.3", features = ["derive"] }
dirs = "5.0"
notify = "5.1"
toml = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
# TUI dependencies
ratatui = "0.23.0"
crossterm = "0.27.0"
tui-widget-list = "0.4"
# Other dependencies
tokio = { version = "1.29", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"
log = "0.4"
env_logger = "0.10"
colored = "2.0"
[[bin]]
name = "workspace"
path = "src/main.rs"
[[bin]]
name = "workspace-daemon"
path = "src/bin/workspace-daemon.rs"
[[bin]]
name = "workspace-monitor"
path = "src/bin/workspace-monitor.rs"
[[bin]]
name = "workspace-changes"
path = "src/bin/workspace-changes.rs"
[[bin]]
name = "workspace-version"
path = "src/bin/workspace-version.rs"
[[bin]]
name = "workspace-graph"
path = "src/bin/workspace-graph.rs"
[lib]
name = "sublime_workspace_cli"
path = "src/lib.rs"
```

## Key Benefits of This Structure

1. **Follows Existing Pattern**: Maintains the `sublime_*` naming convention

2. **Clean Separation**: Separates the CLI from the core library functionality

3. **Multi-Binary Support**: Supports Cargo-style subcommands with separate binaries

4. **Shared Library Components**: Uses a library component for code sharing between binaries

5. **Modular Architecture**: Clearly organizes code by responsibility (daemon, monitor, UI, etc.)

6. **Testability**: Easy to test components in isolation

7. **Asset Management**: Dedicated location for static assets and templates

8. **Example Usage**: Provides examples to help users understand the tool

## Build and Installation Flow

With this structure, the installation process would work as follows:

1. Build all binaries:

    ```bash
    cargo build --release --package sublime_workspace_cli
    ```

2. Install to a path where they're all available:

    ```bash
    cargo install --path sublime_workspace_cli
    ```

3. This would make the main command `workspace` available, which would then be able to discover and execute the subcommands (workspace-daemon, workspace-monitor, etc.)

## Configuration Storage

User configuration would be stored at standard locations:

- **Linux/macOS**: `~/.config/sublime-workspace-cli/config.toml`

- **Windows**: `%APPDATA%\sublime-workspace-cli\config.toml`

Data files would be stored at:

- **Linux/macOS**: `~/.local/share/sublime-workspace-cli/`

- **Windows**: `%LOCALAPPDATA%\sublime-workspace-cli\`

This structure provides a clear, organized, and maintainable approach to implementing the CLI tool while integrating smoothly with your existing monorepo architecture.

