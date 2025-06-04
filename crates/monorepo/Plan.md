# Sublime Monorepo Tools - Plano Final Completo

## Visão Geral e Objetivos (Plan.md)

O `sublime-monorepo-tools` é uma biblioteca central que unifica funcionalidades dos crates base para workflows completos de monorepo Node.js.

### Objetivos Específicos do Plan.md

1. **Versionamento**: Major, Minor, Patch, Snapshot com propagação para dependentes
2. **Diffs**: Reconhecer diferenças entre branches e packages afetados
3. **Tarefas**: Scripts do package.json organizados em tarefas executadas por alterações
4. **Análise do monorepo**: Package manager, dependency graph, packages internos/externos
5. **Monorepo como projeto**: Agregar informações do monorepo
6. **Changelogs**: Baseados em conventional commits com templates
7. **Hooks**: Git hooks (pre-commit, pre-push) com validações
8. **Changesets**: Gestão de changes com ambientes de deploy
9. **Plugins**: Sistema extensível para customização


## Estrutura do Projeto

O projeto será estruturado como um monorepo Rust, reutilizando os crates base existentes. A estrutura será organizada em módulos que correspondem aos objetivos do Plan.md, garantindo máxima reutilização e integração.

```ascii
src/
├── lib.rs                    # Entry point + re-exports
├── config/                   # Configuration management
│   ├── mod.rs
│   ├── types.rs             # Config structs/enums  
│   ├── manager.rs           # ConfigManager implementation
│   ├── defaults.rs          # Default configurations
│   └── tests.rs
├── core/                    # Core project types
│   ├── mod.rs
│   ├── types.rs             # MonorepoProject, PackageInfo
│   ├── project.rs           # MonorepoProject implementation
│   ├── package.rs           # MonorepoPackageInfo implementation  
│   └── tests.rs
├── analysis/                # Dependency & change analysis
│   ├── mod.rs
│   ├── types.rs             # Analysis result types
│   ├── analyzer.rs          # MonorepoAnalyzer implementation
│   ├── change_detector.rs   # Change detection logic
│   └── tests.rs
└── shared/                  # Test utilities
    ├── mod.rs
    ├── mocks.rs
    └── fixtures.rs
```

## Especificação Completa da API

### Core Context e Projeto Monorepo

```rust
/// Projeto monorepo que agrega toda informação (Objetivo 5)
pub struct MonorepoProject {
    // Reutilização direta dos crates base
    repository: Repo,                    // git-tools
    descriptor: MonorepoDescriptor,      // standard-tools
    package_manager: PackageManager,     // standard-tools
    dependency_registry: DependencyRegistry, // package-tools
    registry_manager: RegistryManager,   // package-tools
    command_queue: CommandQueue,         // standard-tools
    config_manager: ConfigManager,       // standard-tools
    file_system: FileSystemManager,     // standard-tools
    
    // Estado específico do monorepo-tools
    packages: Vec<MonorepoPackageInfo>,
    dependency_graph: DependencyGraph<Package>, // package-tools
    config: MonorepoConfig,
}

/// Informação completa de package no contexto do monorepo
pub struct MonorepoPackageInfo {
    package_info: PackageInfo,           // package-tools
    workspace_package: WorkspacePackage, // standard-tools
    is_internal: bool,
    dependents: Vec<String>,
    dependencies_external: Vec<String>,
    version_status: VersionStatus,
    changesets: Vec<Changeset>,
}

/// Configuração completa do monorepo-tools
pub struct MonorepoConfig {
    versioning: VersioningConfig,
    tasks: TasksConfig,
    changelog: ChangelogConfig,
    hooks: HooksConfig,
    changesets: ChangesetsConfig,
    plugins: PluginsConfig,
    environments: Vec<Environment>,
}
```

### 1. Módulo Versionamento (Objetivo 1)

**Interface**: Implementa versionamento completo com propagação

```rust
pub struct VersionManager {
    project: Arc<MonorepoProject>,
    strategy: Box<dyn VersioningStrategy>,
}

impl VersionManager {
    /// Bump usando Version::bump_* do package-tools
    pub async fn bump_package_version(
        &self,
        package_name: &str,
        bump_type: VersionBumpType,
        commit_sha: Option<&str>,
    ) -> Result<VersioningResult>;
    
    /// Propagação automática para dependentes
    pub async fn propagate_version_changes(
        &self,
        updated_package: &str,
    ) -> Result<PropagationResult>;
    
    /// Análise completa de impacto usando DependencyGraph
    pub async fn analyze_version_impact(
        &self,
        changes: &[PackageChange],
    ) -> Result<VersionImpactAnalysis>;
    
    /// Cria plano de versionamento baseado em alterações
    pub async fn create_versioning_plan(
        &self,
        changes: &ChangeAnalysis,
    ) -> Result<VersioningPlan>;
    
    /// Executa plano completo de versionamento
    pub async fn execute_versioning_plan(
        &self,
        plan: &VersioningPlan,
    ) -> Result<VersioningResult>;
}

pub enum VersionBumpType {
    Major,
    Minor, 
    Patch,
    Snapshot, // Com commit SHA
}

pub struct VersioningResult {
    primary_updates: Vec<PackageVersionUpdate>,
    propagated_updates: Vec<PackageVersionUpdate>,
    conflicts: Vec<VersionConflict>,
    dependency_updates: ResolutionResult, // package-tools
}

pub trait VersioningStrategy {
    fn determine_bump_type(&self, changes: &PackageChange) -> VersionBumpType;
    fn should_propagate(&self, bump_type: VersionBumpType) -> bool;
}
```

### 2. Módulo Diffs (Objetivo 2)

**Interface**: Reconhece diferenças entre branches e mapeia para packages

```rust
pub struct DiffAnalyzer {
    project: Arc<MonorepoProject>,
    analyzers: Vec<Box<dyn ChangeAnalyzer>>,
}

impl DiffAnalyzer {
    /// Compara branches usando git-tools
    pub async fn compare_branches(
        &self,
        base_branch: &str,
        target_branch: &str,
    ) -> Result<BranchComparisonResult>;
    
    /// Detecta alterações desde referência usando git-tools
    pub async fn detect_changes_since(
        &self,
        since_ref: &str,
        until_ref: Option<&str>,
    ) -> Result<ChangeAnalysis>;
    
    /// Mapeia arquivos alterados para packages usando MonorepoDescriptor
    pub fn map_changes_to_packages(
        &self,
        changed_files: &[GitChangedFile],
    ) -> Vec<PackageChange>;
    
    /// Identifica packages afetados e dependentes
    pub async fn identify_affected_packages(
        &self,
        changes: &[GitChangedFile],
    ) -> Result<AffectedPackagesAnalysis>;
    
    /// Análise de significância das alterações
    pub fn analyze_change_significance(
        &self,
        package_changes: &[PackageChange],
    ) -> Vec<ChangeSignificanceResult>;
}

pub struct BranchComparisonResult {
    base_branch: String,
    target_branch: String,
    changed_files: Vec<GitChangedFile>, // git-tools
    affected_packages: Vec<String>,
    merge_base: String,
    conflicts: Vec<String>,
}

pub struct ChangeAnalysis {
    from_ref: String,
    to_ref: String,
    changed_files: Vec<GitChangedFile>,
    package_changes: Vec<PackageChange>,
    affected_packages: AffectedPackagesAnalysis,
    significance_analysis: Vec<ChangeSignificanceResult>,
}

pub struct PackageChange {
    package_name: String,
    changed_files: Vec<GitChangedFile>,
    change_type: PackageChangeType,
    significance: ChangeSignificance,
    suggested_version_bump: VersionBumpType,
}

pub enum PackageChangeType {
    SourceCode,
    Dependencies,
    Configuration,
    Documentation,
    Tests,
}
```

### 3. Módulo Tarefas (Objetivo 3)

**Interface**: Organiza scripts package.json em tarefas por alterações

```rust
pub struct TaskManager {
    project: Arc<MonorepoProject>,
    task_registry: TaskRegistry,
}

impl TaskManager {
    /// Executa tarefa usando CommandQueue do standard-tools
    pub async fn execute_task(
        &self,
        task_definition: &TaskDefinition,
        scope: TaskScope,
    ) -> Result<TaskExecutionResult>;
    
    /// Executa tarefas baseado em packages afetados
    pub async fn execute_tasks_for_affected_packages(
        &self,
        affected_packages: &[String],
    ) -> Result<Vec<TaskExecutionResult>>;
    
    /// Executa tarefas baseado em alterações detectadas
    pub async fn execute_tasks_for_changes(
        &self,
        changes: &ChangeAnalysis,
    ) -> Result<Vec<TaskExecutionResult>>;
    
    /// Executa batch de tarefas usando CommandQueue batch
    pub async fn execute_tasks_batch(
        &self,
        tasks: &[(TaskDefinition, TaskScope)],
    ) -> Result<Vec<TaskExecutionResult>>;
    
    /// Resolve tarefas do package.json para TaskDefinitions
    pub fn resolve_package_tasks(
        &self,
        package_name: &str,
    ) -> Result<Vec<TaskDefinition>>;
    
    /// Registra tarefas personalizadas
    pub fn register_task(&mut self, task: TaskDefinition);
}

pub struct TaskDefinition {
    name: String,
    description: String,
    commands: Vec<TaskCommand>,
    package_scripts: Vec<PackageScript>, // Scripts do package.json
    dependencies: Vec<String>,
    conditions: Vec<TaskCondition>,
    priority: CommandPriority, // standard-tools
    scope: TaskScope,
}

pub struct PackageScript {
    package_name: String,
    script_name: String, // npm run test, build, etc.
    working_directory: PathBuf,
}

pub struct TaskCommand {
    program: String,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    env: HashMap<String, String>,
    timeout: Option<Duration>,
}

pub enum TaskCondition {
    PackagesChanged { packages: Vec<String> },
    FilesChanged { patterns: Vec<String> },
    DependenciesChanged,
    OnBranch { pattern: String },
    Environment { env: Environment },
}

pub enum TaskScope {
    Global,
    Package(String),
    AffectedPackages,
    AllPackages,
}
```

### 4. Módulo Análise Monorepo (Objetivo 4)

**Interface**: Informação completa do monorepo usando crates base

```rust
pub struct MonorepoAnalyzer {
    project: Arc<MonorepoProject>,
}

impl MonorepoAnalyzer {
    /// Detecção completa usando MonorepoDetector
    pub async fn detect_monorepo_info(
        &self,
        path: &Path,
    ) -> Result<MonorepoAnalysisResult>;
    
    /// Análise de package manager usando PackageManager
    pub async fn analyze_package_manager(&self) -> Result<PackageManagerAnalysis>;
    
    /// Constrói dependency graph usando package-tools
    pub async fn build_dependency_graph(&self) -> Result<DependencyGraphAnalysis>;
    
    /// Identifica packages internos vs externos
    pub fn classify_packages(&self) -> PackageClassificationResult;
    
    /// Análise de registry usando RegistryManager
    pub async fn analyze_registries(&self) -> Result<RegistryAnalysisResult>;
    
    /// Informações completas de packages
    pub fn get_package_information(&self) -> Vec<PackageInformation>;
    
    /// Análise de upgrades usando Upgrader
    pub async fn analyze_available_upgrades(&self) -> Result<UpgradeAnalysisResult>;
}

pub struct MonorepoAnalysisResult {
    kind: MonorepoKind,           // standard-tools
    root_path: PathBuf,
    package_manager: PackageManagerAnalysis,
    packages: PackageClassificationResult,
    dependency_graph: DependencyGraphAnalysis,
    registries: RegistryAnalysisResult,
    workspace_config: WorkspaceConfigAnalysis,
}

pub struct PackageManagerAnalysis {
    kind: PackageManagerKind,     // standard-tools
    version: String,
    lock_file: PathBuf,
    config_files: Vec<PathBuf>,
    workspaces_config: Value,
}

pub struct PackageClassificationResult {
    internal_packages: Vec<PackageInformation>,
    external_dependencies: Vec<String>,
    dev_dependencies: Vec<String>,
    peer_dependencies: Vec<String>,
}

pub struct PackageInformation {
    name: String,
    version: String,
    path: PathBuf,
    relative_path: PathBuf,
    package_json: Value,
    is_internal: bool,
    dependencies: Vec<String>,
    dev_dependencies: Vec<String>,
    workspace_dependencies: Vec<String>,
    dependents: Vec<String>,
}
```

### 5. Módulo Changesets (Objetivo 8)

**Interface**: Gestão completa de changes com ambientes

```rust
pub struct ChangesetManager {
    project: Arc<MonorepoProject>,
    storage: Box<dyn ChangesetStorage>,
}

impl ChangesetManager {
    /// Cria changeset com ambientes de desenvolvimento
    pub async fn create_changeset(
        &self,
        spec: ChangesetSpec,
    ) -> Result<Changeset>;
    
    /// Cria changeset interativo com prompts
    pub async fn create_changeset_interactive(
        &self,
        package: Option<String>,
    ) -> Result<Changeset>;
    
    /// Aplica changesets no merge para todos ambientes
    pub async fn apply_changesets_on_merge(
        &self,
        branch: &str,
    ) -> Result<Vec<ChangesetApplication>>;
    
    /// Lista changesets por filtros
    pub async fn list_changesets(
        &self,
        filter: ChangesetFilter,
    ) -> Result<Vec<Changeset>>;
    
    /// Valida changeset antes de aplicar
    pub fn validate_changeset(&self, changeset: &Changeset) -> Result<ValidationResult>;
    
    /// Deploy para ambientes específicos durante desenvolvimento
    pub async fn deploy_to_environments(
        &self,
        changeset_id: &str,
        environments: &[Environment],
    ) -> Result<DeploymentResult>;
}

pub struct Changeset {
    id: String,
    package: String,
    version_bump: VersionBumpType,
    description: String,
    branch: String,
    development_environments: Vec<Environment>, // stage, dev, int
    production_deployment: bool,
    created_at: DateTime<Utc>,
    author: String,
    status: ChangesetStatus,
}

pub enum Environment {
    Development,
    Staging,
    Integration, 
    Production,
    Custom(String),
}

pub enum ChangesetStatus {
    Pending,
    PartiallyDeployed { environments: Vec<Environment> },
    FullyDeployed { deployed_at: DateTime<Utc> },
    Merged { merged_at: DateTime<Utc>, final_version: String },
}

pub struct ChangesetApplication {
    changeset_id: String,
    package: String,
    old_version: String,
    new_version: String,
    environments_deployed: Vec<Environment>,
    success: bool,
}
```

### 6. Módulo Hooks (Objetivo 7)

**Interface**: Gestão de Git hooks com validações

```rust
pub struct HookManager {
    project: Arc<MonorepoProject>,
}

impl HookManager {
    /// Instala hooks usando FileSystemManager
    pub async fn install_hooks(&self) -> Result<Vec<HookType>>;
    
    /// Executa hook com validações
    pub async fn execute_hook(
        &self,
        hook_type: HookType,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult>;
    
    /// Pre-commit: valida se changeset existe
    pub async fn pre_commit_validation(&self) -> Result<PreCommitResult>;
    
    /// Pre-push: executa tarefas nos packages afetados  
    pub async fn pre_push_validation(
        &self,
        pushed_commits: &[String],
    ) -> Result<PrePushResult>;
    
    /// Prompt para criar changeset se não existe
    pub async fn prompt_for_changeset(&self) -> Result<Changeset>;
    
    /// Configura hooks personalizados
    pub fn configure_hook(&mut self, hook_type: HookType, definition: HookDefinition);
}

pub enum HookType {
    PreCommit,
    PrePush,
    PostCommit,
    PostMerge,
    PostCheckout,
}

pub struct HookDefinition {
    script: HookScript,
    conditions: Vec<HookCondition>,
    fail_on_error: bool,
    timeout: Option<Duration>,
}

pub enum HookScript {
    TaskExecution { tasks: Vec<String> },
    Command { cmd: String, args: Vec<String> },
    ScriptFile { path: PathBuf },
}

pub struct PreCommitResult {
    changeset_exists: bool,
    validation_passed: bool,
    required_actions: Vec<String>,
}
```

### 7. Módulo Changelog (Objetivo 6)

**Interface**: Geração baseada em conventional commits

```rust
pub struct ChangelogManager {
    project: Arc<MonorepoProject>,
    config: ChangelogConfig,
}

impl ChangelogManager {
    /// Gera changelog para package usando git-tools
    pub async fn generate_package_changelog(
        &self,
        package_name: &str,
        from_version: Option<&str>,
        to_version: Option<&str>,
    ) -> Result<GeneratedChangelog>;
    
    /// Gera changelogs para packages afetados
    pub async fn generate_changelogs_for_affected(
        &self,
        affected_packages: &[String],
        version_updates: &[PackageVersionUpdate],
    ) -> Result<Vec<GeneratedChangelog>>;
    
    /// Parse conventional commits usando git-tools
    pub async fn parse_conventional_commits(
        &self,
        package_path: Option<&str>,
        since: &str,
    ) -> Result<Vec<ConventionalCommit>>;
    
    /// Atualiza changelog existente
    pub async fn update_existing_changelog(
        &self,
        package_name: &str,
        new_version: &str,
        changelog_path: &Path,
    ) -> Result<()>;
    
    /// Renderiza usando templates
    pub fn render_changelog_with_template(
        &self,
        commits: &[ConventionalCommit],
        template: &ChangelogTemplate,
    ) -> Result<String>;
}

pub struct ChangelogConfig {
    template: ChangelogTemplate,
    grouping: CommitGrouping,
    output_format: ChangelogFormat,
    include_breaking_changes: bool,
    conventional_commit_types: HashMap<String, String>,
}

pub struct ChangelogTemplate {
    header_template: String,
    section_template: String,
    commit_template: String,
    footer_template: String,
}

pub struct ConventionalCommit {
    hash: String,
    commit_type: String, // feat, fix, docs, etc.
    scope: Option<String>,
    description: String,
    body: Option<String>,
    breaking_changes: Vec<String>,
    affected_packages: Vec<String>,
    author: String,
    date: DateTime<Utc>,
}
```

### 8. Módulo Plugins (Objetivo 9)

**Interface**: Sistema extensível para customização

```rust
pub struct PluginManager {
    project: Arc<MonorepoProject>,
    loaded_plugins: HashMap<String, Box<dyn MonorepoPlugin>>,
    registry: PluginRegistry,
}

impl PluginManager {
    /// Carrega plugin por nome
    pub async fn load_plugin(&mut self, plugin_name: &str) -> Result<()>;
    
    /// Executa comando em plugin
    pub async fn execute_plugin_command(
        &self,
        plugin_name: &str,
        command: &str,
        args: &[String],
    ) -> Result<PluginExecutionResult>;
    
    /// Gera package skeleton usando template plugin
    pub async fn generate_package_skeleton(
        &self,
        package_name: &str,
        template: &str,
        options: &HashMap<String, String>,
    ) -> Result<()>;
    
    /// Aplica change analyzers personalizados
    pub async fn apply_change_analyzers(
        &self,
        changes: &[GitChangedFile],
    ) -> Result<Vec<ChangeSignificance>>;
    
    /// Registra plugin personalizado
    pub fn register_plugin(&mut self, plugin: Box<dyn MonorepoPlugin>);
}

pub trait MonorepoPlugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn initialize(&mut self, context: &PluginContext) -> Result<()>;
    fn execute_command(&self, command: &str, args: &[String]) -> Result<Value>;
}

pub trait TemplateGeneratorPlugin: MonorepoPlugin {
    fn generate_package_skeleton(
        &self,
        name: &str,
        template: &str,
        options: &HashMap<String, String>,
    ) -> Result<()>;
    
    fn list_templates(&self) -> Vec<TemplateInfo>;
}

pub trait ChangeAnalyzerPlugin: MonorepoPlugin {
    fn analyze_change(&self, change: &GitChangedFile) -> Result<ChangeSignificance>;
}
```

### 9. API Principal MonorepoTools

**Interface**: Orquestração completa de workflows

```rust
pub struct MonorepoTools {
    project: Arc<MonorepoProject>,
    version_manager: VersionManager,
    diff_analyzer: DiffAnalyzer,
    task_manager: TaskManager,
    analyzer: MonorepoAnalyzer,
    changeset_manager: ChangesetManager,
    hook_manager: HookManager,
    changelog_manager: ChangelogManager,
    plugin_manager: PluginManager,
}

impl MonorepoTools {
    /// Inicializa usando detectors dos crates base
    pub async fn initialize(path: impl AsRef<Path>) -> Result<Self>;
    
    /// Abre projeto existente
    pub async fn open(path: impl AsRef<Path>) -> Result<Self>;
    
    /// Workflows principais
    
    /// Workflow completo de release
    pub async fn release_workflow(
        &self,
        options: ReleaseOptions,
    ) -> Result<ReleaseResult>;
    
    /// Workflow de desenvolvimento
    pub async fn development_workflow(
        &self,
        since: Option<&str>,
    ) -> Result<DevelopmentResult>;
    
    /// Workflow de análise de alterações
    pub async fn analyze_changes_workflow(
        &self,
        from_branch: &str,
        to_branch: Option<&str>,
    ) -> Result<ChangeAnalysisResult>;
    
    /// Workflow de versionamento
    pub async fn versioning_workflow(
        &self,
        plan: Option<VersioningPlan>,
    ) -> Result<VersioningWorkflowResult>;
    
    /// Informações do projeto (Objetivo 5)
    pub fn project_info(&self) -> &MonorepoProject;
    pub fn list_packages(&self) -> &[MonorepoPackageInfo];
    pub fn get_package_info(&self, name: &str) -> Option<&MonorepoPackageInfo>;
    pub fn get_dependency_graph(&self) -> &DependencyGraph<Package>;
    pub async fn project_status(&self) -> Result<MonorepoStatus>;
}

pub struct ReleaseResult {
    changes: ChangeAnalysis,
    versioning: VersioningResult,
    tasks: Vec<TaskExecutionResult>,
    changelogs: Vec<GeneratedChangelog>,
    changesets_applied: Vec<ChangesetApplication>,
    duration: Duration,
}

pub struct DevelopmentResult {
    changes: ChangeAnalysis,
    affected_tasks: Vec<TaskExecutionResult>,
    changeset_status: ChangesetStatus,
    recommendations: Vec<String>,
}
```

## Fluxos de Execução Principais

### 1. Fluxo Release Workflow Completo

```
1. DiffAnalyzer::detect_changes_since(last_tag)
   ↓ usa git-tools: Repo::get_all_files_changed_since_sha
2. DiffAnalyzer::map_changes_to_packages()
   ↓ usa standard-tools: MonorepoDescriptor::find_package_for_path
3. VersionManager::create_versioning_plan()
   ↓ usa package-tools: DependencyGraph para análise de impacto
4. ChangesetManager::apply_changesets_on_merge()
   ↓ aplica changesets pendentes
5. VersionManager::execute_versioning_plan()
   ↓ usa package-tools: Version::bump_* + propagação
6. TaskManager::execute_tasks_for_affected_packages()
   ↓ usa standard-tools: CommandQueue para execução
7. ChangelogManager::generate_changelogs_for_affected()
   ↓ usa git-tools: Repo::get_commits_since
8. Deploy para todos ambientes (Production)
```

### 2. Fluxo Development Workflow

```
1. DiffAnalyzer::detect_changes_since(HEAD~1)
2. ChangesetManager::list_changesets(current_branch)
3. HookManager::pre_commit_validation()
   ↓ verifica se changeset existe para alterações
4. TaskManager::execute_tasks_for_changes()
   ↓ executa testes/lint nos packages afetados
5. Se changeset não existe → prompt para criação
```

### 3. Fluxo Pre-Commit Hook

```
1. HookManager::execute_hook(PreCommit)
2. DiffAnalyzer::detect_changes_since(HEAD)
3. ChangesetManager::validate_changeset_exists()
4. Se não existe → HookManager::prompt_for_changeset()
5. TaskManager::execute_tasks_for_affected_packages()
6. Retorna sucesso/falha para Git
```

### 4. Fluxo Análise Completa Monorepo

```
1. MonorepoAnalyzer::detect_monorepo_info()
   ↓ usa standard-tools: MonorepoDetector
2. MonorepoAnalyzer::analyze_package_manager()
   ↓ usa standard-tools: PackageManager
3. MonorepoAnalyzer::build_dependency_graph()
   ↓ usa package-tools: DependencyGraph
4. MonorepoAnalyzer::analyze_registries()
   ↓ usa package-tools: RegistryManager
5. MonorepoAnalyzer::analyze_available_upgrades()
   ↓ usa package-tools: Upgrader
```

## Estratégia de Implementação

### Fase 1: Projeto e Context (2 semanas)
- **MonorepoProject**: Composição completa dos crates base
- **MonorepoAnalyzer**: Análise completa usando detectores
- **Error hierarchy**: Propagação de todos os errors
- **Testes**: Detecção e inicialização de projetos reais

### Fase 2: Diffs e Versionamento (3 semanas)
- **DiffAnalyzer**: Comparação branches + mapeamento packages
- **VersionManager**: Wrapper completo sobre package-tools
- **Propagação**: Sistema completo de dependency updates
- **Testes**: Cenários complexos de versionamento

### Fase 3: Tasks e Hooks (2 semanas)
- **TaskManager**: Execução baseada em package.json scripts
- **HookManager**: Instalação e execução com validações
- **Integração**: Tasks baseadas em alterações detectadas

### Fase 4: Changesets e Workflows (3 semanas)
- **ChangesetManager**: Sistema completo com ambientes
- **Workflows**: Release e development workflows
- **Storage**: Persistência usando FileSystemManager

### Fase 5: Changelogs e Plugins (2 semanas)
- **ChangelogManager**: Geração com templates
- **PluginManager**: Sistema extensível básico
- **Templates**: Sistema de geração de packages

### Fase 6: API Principal e Polish (2 semanas)
- **MonorepoTools**: Interface principal completa
- **Workflows integrados**: Todos os fluxos funcionando
- **Documentação**: API completa e exemplos
- **Performance**: Otimizações e benchmarks

## Critérios de Sucesso

- **Cobertura objetivos**: 100% dos objetivos Plan.md implementados
- **Reutilização**: 80%+ funcionalidades dos crates base
- **Performance**: Release workflow < 45s (monorepos 30+ packages)
- **Extensibilidade**: Sistema plugins funcional
- **Usabilidade**: APIs simples para casos comuns

Este plano cumpre integralmente todos os objetivos do Plan.md usando máxima reutilização dos crates base.