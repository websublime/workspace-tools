# Plano: workspace status & workspace execute

## Resumo

Dois novos comandos CLI:
1. **`workspace status`** - Info do workspace (tipo repo, package manager, changesets, branch, packages)
2. **`workspace execute`** - Executa comandos nos packages com filtros

## Decisões de Design

| Aspecto | Decisão |
|---------|---------|
| Execução múltiplos packages | Sequencial por defeito, `--parallel` para paralelo |
| Comportamento em falha | Continuar execução, reportar falhas no final |
| Script npm inexistente | Falhar com erro claro |
| Output de comandos | Streaming em tempo real |
| Formato packages (status) | Tabela: Name, Version, Path |
| Info changesets | Apenas ID/branch |

## Estrutura de Ficheiros

### Novos Ficheiros
```
crates/cli/src/commands/
├── status.rs       # Comando status (inclui tipos JSON internos)
└── execute.rs      # Comando execute (inclui tipos JSON internos)
```

### Ficheiros a Modificar
| Ficheiro | Alteração |
|----------|-----------|
| `crates/cli/src/commands/mod.rs` | Adicionar `pub mod status;` e `pub mod execute;` |
| `crates/cli/src/cli/commands.rs` | Adicionar `WorkspaceCommands` enum e args structs |
| `crates/cli/src/cli/dispatch.rs` | Adicionar routing para workspace commands |
| `crates/cli/src/commands/tests.rs` | Adicionar testes para status e execute |

**Nota:** Seguindo o padrão de `changes.rs`, os tipos JSON são definidos dentro de cada ficheiro de comando (não há `types.rs` separado).

---

## Definição de Comandos (cli/commands.rs)

```rust
/// Workspace management commands.
#[derive(Debug, Subcommand)]
pub enum WorkspaceCommands {
    /// Display workspace status information.
    Status(WorkspaceStatusArgs),
    /// Execute commands across workspace packages.
    Execute(WorkspaceExecuteArgs),
}

/// Arguments for workspace status command.
#[derive(Debug, Args)]
pub struct WorkspaceStatusArgs {
    // Usa opções globais (--format, --root, etc.)
}

/// Arguments for workspace execute command.
#[derive(Debug, Args)]
pub struct WorkspaceExecuteArgs {
    /// Command to execute.
    /// Prefixes: `npm:<script>` for npm scripts, or plain commands for system execution.
    #[arg(long, value_name = "COMMAND")]
    pub cmd: String,

    /// Filter packages to run command on (comma-separated package names).
    #[arg(long = "filter-package", value_name = "PACKAGES", value_delimiter = ',')]
    pub filter_package: Option<Vec<String>>,

    /// Run commands in parallel across packages.
    #[arg(long, default_value = "false")]
    pub parallel: bool,

    /// Additional arguments passed to the command.
    #[arg(last = true)]
    pub args: Vec<String>,
}

// Adicionar ao Commands enum existente:
#[command(subcommand)]
Workspace(WorkspaceCommands),
```

---

## Feature 1: workspace status

### Ficheiro: `crates/cli/src/commands/status.rs`

### Fluxo de Implementação
```
1. Detectar tipo de projeto
   -> MonorepoDetector::detect_monorepo() ou ProjectDetector
   -> RepoKind (Simple/Monorepo)

2. Detectar package manager
   -> PackageManager::detect(root)
   -> PackageManagerKind (npm/yarn/pnpm/bun)

3. Obter branch Git (graceful degradation)
   -> Repo::open(root).get_current_branch()
   -> Option<String>

4. Listar changesets pendentes
   -> ChangesetManager::new().list_pending()
   -> Vec<Changeset> (extrair apenas branch/id)

5. Listar packages
   -> MonorepoDescriptor::packages()
   -> Vec<WorkspacePackage>
   -> Para repo simples: ler package.json root

6. Formatar output
   -> Human: secções + tabela
   -> JSON: JsonResponse<StatusJsonResponse>
```

### Tipos JSON (dentro de status.rs)

```rust
/// JSON response for workspace status command.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusJsonResponse {
    repository: RepositoryInfoJson,
    package_manager: PackageManagerInfoJson,
    branch: Option<BranchInfoJson>,
    changesets: Vec<ChangesetInfoJson>,
    packages: Vec<PackageInfoJson>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryInfoJson {
    kind: String,                    // "simple" ou "monorepo"
    monorepo_type: Option<String>,   // "npm", "yarn", "pnpm", "bun", etc.
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageManagerInfoJson {
    name: String,
    lock_file: String,
}

#[derive(Debug, Clone, Serialize)]
struct BranchInfoJson {
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChangesetInfoJson {
    id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageInfoJson {
    name: String,
    version: String,
    path: String,
}
```

### Output Human Format
```
Workspace Status
================

Repository
  Type: monorepo (pnpm)

Package Manager
  Name: pnpm
  Lock file: pnpm-lock.yaml

Git Branch
  Current: feature/workspace-commands

Active Changesets (2)
  - feat/new-api
  - fix/validation-bug

Packages (5)
| Name             | Version | Path            |
|------------------|---------|-----------------|
| @scope/core      | 1.2.3   | packages/core   |
| @scope/utils     | 0.5.0   | packages/utils  |
```

### Tratamento de Erros (status)

| Cenário | Comportamento |
|---------|---------------|
| Não é repositório Git | Mostrar status sem branch (continuar) |
| Não encontra package.json | Erro: "Not a valid Node.js project" |
| Directório changesets não existe | Lista vazia de changesets |
| Package manager não detectado | Warning, usar "unknown" |

---

## Feature 2: workspace execute

### Ficheiro: `crates/cli/src/commands/execute.rs`

### Fluxo de Implementação
```
1. Parse do comando
   -> "npm:lint" => CommandType::NpmScript { script: "lint" }
   -> "node index.js" => CommandType::System { program: "node", args: ["index.js"] }
   -> "./scripts/build.sh" => CommandType::System { program: "./scripts/build.sh", args: [] }

2. Obter packages alvo
   -> MonorepoDetector::detect_packages()
   -> Vec<WorkspacePackage>

3. Aplicar filtro (se --filter-package)
   -> Filtrar por nome de package
   -> Erro se nenhum package corresponder

4. Para npm scripts: Validar existência
   -> Ler package.json de cada package
   -> Verificar se script existe em "scripts"
   -> ERRO se script não existir

5. Executar comandos
   -> Sequencial por defeito, paralelo se --parallel
   -> Streaming output em tempo real
   -> Capturar exit codes

6. Continuar em caso de falha
   -> Acumular resultados (success/failed)
   -> Reportar sumário no final

7. Retornar exit code
   -> 0 se todos succeeded
   -> 1 se algum falhou
```

### Tipos (dentro de execute.rs)

```rust
/// Type of command to execute.
#[derive(Debug, Clone)]
enum CommandType {
    /// npm script (e.g., npm:build -> npm run build)
    NpmScript { script: String },
    /// System command (e.g., node index.js)
    System { program: String, args: Vec<String> },
}

impl CommandType {
    /// Parse command string into CommandType.
    fn parse(cmd: &str) -> Self {
        if let Some(script) = cmd.strip_prefix("npm:") {
            CommandType::NpmScript { script: script.to_string() }
        } else {
            let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
            let program = parts[0].to_string();
            let args = parts.get(1)
                .map(|s| shell_words::split(s).unwrap_or_default())
                .unwrap_or_default();
            CommandType::System { program, args }
        }
    }
}

/// JSON response for execute command.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteJsonResponse {
    command: String,
    results: Vec<PackageExecutionResultJson>,
    summary: ExecuteSummaryJson,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageExecutionResultJson {
    package: String,
    success: bool,
    exit_code: i32,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteSummaryJson {
    total: usize,
    succeeded: usize,
    failed: usize,
    total_duration_ms: u64,
}
```

### Output Human Format
```
Executing: npm run lint

> @scope/core (lint)
[streaming output...]

> @scope/utils (lint)
[streaming output...]

> @scope/api (lint)
[streaming output...]
error: ESLint found 3 errors

----------------------------------------
Summary
  Total: 3 | Succeeded: 2 | Failed: 1
  Duration: 4.2s

Failed packages:
  - @scope/api (exit code: 1)
```

### Tratamento de Erros (execute)

| Cenário | Comportamento |
|---------|---------------|
| Script npm não existe | Erro imediato: "Script 'X' not found in package 'Y'" |
| Comando falha (exit != 0) | Registar falha, continuar para próximo package |
| Nenhum package no filtro | Erro: "No packages match filter: X, Y" |
| Não é monorepo | Executar no package root (se existir) |

---

## Dispatch (cli/dispatch.rs)

```rust
Commands::Workspace(workspace_cmd) => {
    match workspace_cmd {
        WorkspaceCommands::Status(args) => {
            crate::commands::status::execute_status(
                args,
                &output,
                root,
                config_path.as_ref().map(|p| p.as_path()),
            ).await?;
        }
        WorkspaceCommands::Execute(args) => {
            crate::commands::execute::execute_execute(
                args,
                &output,
                root,
                config_path.as_ref().map(|p| p.as_path()),
            ).await?;
        }
    }
}
```

---

## APIs Existentes a Usar

### sublime_standard_tools
| API | Uso |
|-----|-----|
| `MonorepoDetector::detect_monorepo(path)` | Detectar tipo de monorepo |
| `MonorepoDescriptor::kind()` | Obter MonorepoKind |
| `MonorepoDescriptor::packages()` | Listar WorkspacePackage |
| `PackageManager::detect(path)` | Detectar package manager |
| `PackageManagerKind::command()` | Obter comando (npm, yarn, etc.) |
| `PackageManagerKind::lock_file()` | Obter nome do lock file |
| `CommandBuilder::new().arg().current_dir().build()` | Construir comandos |
| `DefaultCommandExecutor::execute_stream()` | Executar com streaming |
| `FileSystemManager::read_file_string()` | Ler package.json |

### sublime_git_tools
| API | Uso |
|-----|-----|
| `Repo::open(path)` | Abrir repositório |
| `Repo::get_current_branch()` | Obter branch actual |

### sublime_pkg_tools
| API | Uso |
|-----|-----|
| `ChangesetManager::new()` | Criar manager |
| `ChangesetManager::list_pending()` | Listar changesets activos |
| `Changeset::branch` | ID/nome do changeset |

---

## Sequência de Implementação

### Fase 1: Estrutura Base
1. Adicionar `WorkspaceCommands` enum e args a `cli/commands.rs`
2. Adicionar routing a `cli/dispatch.rs`
3. Adicionar `pub mod status;` e `pub mod execute;` a `commands/mod.rs`

### Fase 2: workspace status
1. Criar `commands/status.rs`
2. Implementar `execute_status()` seguindo padrão de `changes.rs`
3. Integrar com MonorepoDetector, PackageManager, Repo, ChangesetManager
4. Implementar formatação Human (tabelas) e JSON
5. Adicionar testes a `commands/tests.rs`

### Fase 3: workspace execute
1. Criar `commands/execute.rs`
2. Implementar `CommandType::parse()`
3. Implementar validação de npm scripts (ler package.json)
4. Implementar execução com streaming
5. Implementar modo paralelo (`--parallel`)
6. Implementar sumário de resultados
7. Adicionar testes a `commands/tests.rs`

### Fase 4: Finalização
1. Testes de integração E2E
2. Documentação (doc comments)
3. Verificação clippy 100%
4. Verificação cobertura testes

---

## Ficheiros Críticos a Ler Antes de Implementar

| Ficheiro | Razão |
|----------|-------|
| `crates/cli/src/commands/changes.rs` | Padrão de referência para estrutura e tipos |
| `crates/cli/src/output/mod.rs` | API de Output (json, table, plain) |
| `crates/cli/src/output/table.rs` | TableBuilder API |
| `crates/standard/src/monorepo/detector.rs` | MonorepoDetector API |
| `crates/standard/src/command/executor.rs` | Execução de comandos |
| `crates/standard/src/command/stream.rs` | Streaming output |
| `crates/pkg/src/changeset/manager.rs` | ChangesetManager API |
| `crates/git/src/repo.rs` | Repo API para Git |

---

## Estimativa de Linhas

| Ficheiro | Linhas Estimadas |
|----------|------------------|
| `commands/status.rs` | ~300 |
| `commands/execute.rs` | ~400 |
| Modificações em `cli/commands.rs` | ~40 |
| Modificações em `cli/dispatch.rs` | ~20 |
| Modificações em `commands/mod.rs` | ~5 |
| Testes em `commands/tests.rs` | ~250 |
| **Total** | **~1015** |
