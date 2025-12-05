# PLAN_NODE.md - Plano de Implementação de Node.js Bindings

**Projeto**: workspace-node-tools  
**Objectivo**: Implementar bindings Node.js (napi-rs) para 23 funções execute do CLI  
**Tecnologia**: napi-rs v3.x  
**Data**: 2025-01-20  
**Status**: Planning  
**Referência**: [NAPI_RESEARCH.md](./NAPI_RESEARCH.md)

---

## ÍNDICE

1. [Executive Summary](#1-executive-summary)
2. [Estado Actual](#2-estado-actual)
3. [Arquitectura Proposta](#3-arquitectura-proposta)
4. [Fases de Implementação](#4-fases-de-implementação)
5. [Especificações Técnicas Detalhadas](#5-especificações-técnicas-detalhadas)
6. [Tarefas por Sprint](#6-tarefas-por-sprint)
7. [Critérios de Aceitação](#7-critérios-de-aceitação)
8. [Riscos e Mitigações](#8-riscos-e-mitigações)
9. [Checklist de Qualidade](#9-checklist-de-qualidade)

---

## 1. EXECUTIVE SUMMARY

### 1.1 Objectivo

Criar o crate `crates/node/` com bindings napi-rs que exponham 23 funções `execute_*` do CLI como funções JavaScript/TypeScript nativas, retornando objectos JS estruturados com tipos TypeScript gerados automaticamente.

### 1.2 Decisões Arquitecturais Tomadas

| Decisão | Escolha | Rationale |
|---------|---------|-----------|
| Return Type | Objectos JS nativos via `#[napi(object)]` | Type-safe, sem `JSON.parse()`, DX superior |
| Error Handling | `ApiResponse<T>` + `ErrorInfo` com códigos | Consistente, programático, Node.js style |
| Validação | Camada NAPI (fail fast) | Erros claros antes de invocar CLI |
| Timeout (Execute) | Config default + override por parâmetro | Flexibilidade máxima |
| Parâmetros CLI-only | Valores fixos (`non_interactive=true`) | API é programática, sem prompts |

### 1.3 Funções a Implementar (23)

| Categoria | Funções | Prioridade |
|-----------|---------|------------|
| Status | `status` | P0 (POC) |
| Init | `init` | P1 |
| Config | `configShow`, `configValidate` | P1 |
| Changeset | `changesetAdd`, `changesetUpdate`, `changesetList`, `changesetShow`, `changesetRemove`, `changesetHistory`, `changesetCheck` | P1 |
| Bump | `bumpPreview`, `bumpApply`, `bumpSnapshot` | P1 |
| Upgrade | `upgradeCheck`, `upgradeApply`, `backupList`, `backupRestore`, `backupClean` | P2 |
| Execute | `execute` | P1 |
| Audit | `audit` | P2 |
| Changes | `changes` | P2 |
| Clone | `clone` | P2 |

### 1.4 Timeline Estimado

**Total**: 7-8 semanas

| Fase | Semanas | Descrição |
|------|---------|-----------|
| Foundation | 1 | Criar crate, tipos base, error handling |
| POC | 1 | status + init + testes básicos |
| Core Commands | 2 | Changeset + Bump |
| Execute & Config | 1 | Execute com timeout + Config |
| Remaining | 1-2 | Upgrade, Audit, Changes, Clone |
| Polish | 1 | Testes 100%, docs, CI/CD |

---

## 2. ESTADO ACTUAL

### 2.1 Estrutura de Directórios Existente

```
workspace-node-tools/
├── crates/
│   ├── cli/              # CLI com 23 execute_* functions
│   ├── git/              # Git tools
│   ├── pkg/              # Package tools (changeset, bump, etc)
│   └── standard/         # Standard tools (workspace detection)
├── packages/
│   └── workspace-tools/  # Package NPM EXISTENTE (v1.0.2)
│       ├── src/
│       │   ├── binding.d.ts   # TypeScript defs GERADAS
│       │   ├── binding.js     # JavaScript wrapper GERADO
│       │   └── index.ts       # Entry point (exports existentes)
│       └── package.json       # @websublime/workspace-tools
└── Cargo.toml                 # Workspace root
```

### 2.2 Package NPM Existente

**Package**: `@websublime/workspace-tools` v1.0.2

**Exports Actuais**:
- `MonorepoProject` (classe)
- `MonorepoRepository` (classe)
- `GitCommit`, `GitTag`, `GitChangedFile`, `GitFileStatus`
- `getVersion()` function

**Nota**: A infraestrutura napi-rs já está funcional. O package.json já referencia `crates/node/Cargo.toml` nos scripts de build, mas o crate ainda não existe.

### 2.3 Workspace Cargo.toml

```toml
[workspace]
members = ["crates/*"]

[workspace.dependencies]
sublime_cli_tools = { version = "0.0.23", path = "crates/cli" }
sublime_git_tools = { version = "0.0.14", path = "crates/git" }
sublime_pkg_tools = { version = "0.0.18", path = "crates/pkg" }
sublime_standard_tools = { version = "0.0.14", path = "crates/standard" }
```

---

## 3. ARQUITECTURA PROPOSTA

### 3.1 Estrutura do Novo Crate

```
crates/node/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs                  # Entry point, exports NAPI
│   ├── error.rs                # ErrorInfo, error codes, conversão
│   ├── validation.rs           # Validadores de parâmetros
│   ├── response.rs             # ApiResponse<T> wrapper
│   ├── types/                  # Structs NAPI (params + responses)
│   │   ├── mod.rs
│   │   ├── common.rs           # Tipos partilhados
│   │   ├── status.rs
│   │   ├── init.rs
│   │   ├── config.rs
│   │   ├── changeset.rs
│   │   ├── bump.rs
│   │   ├── upgrade.rs
│   │   ├── audit.rs
│   │   ├── changes.rs
│   │   ├── clone.rs
│   │   └── execute.rs
│   └── commands/               # Implementação das funções NAPI
│       ├── mod.rs
│       ├── status.rs
│       ├── init.rs
│       ├── config.rs
│       ├── changeset.rs
│       ├── bump.rs
│       ├── upgrade.rs
│       ├── audit.rs
│       ├── changes.rs
│       ├── clone.rs
│       └── execute.rs
└── tests/
    ├── error_tests.rs
    └── validation_tests.rs
```

### 3.2 Fluxo de Dados

```
┌─────────────────────────────────────────────────────────────────────┐
│                         JavaScript/TypeScript                        │
│                                                                      │
│   const result = await status({ root: "/path/to/project" });        │
│                                                                      │
│   if (result.success) {                                             │
│     console.log(result.data.packages);  // ← Objectos JS nativos    │
│   } else {                                                          │
│     console.error(result.error.code);   // ← "EVALIDATION"          │
│   }                                                                 │
└────────────────────────────────────┬────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      NAPI Layer (crates/node/)                       │
│                                                                      │
│   1. Validação de parâmetros (validators.rs)                        │
│   2. Conversão JS → Rust structs                                    │
│   3. Chamada de execute_* functions do CLI                          │
│   4. Captura de output JSON                                         │
│   5. Parsing e conversão para structs NAPI                          │
│   6. Retorno de ApiResponse<T>                                      │
└────────────────────────────────────┬────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      CLI Layer (crates/cli/)                         │
│                                                                      │
│   execute_status(), execute_init(), execute_changeset_add(), ...    │
│                                                                      │
│   - Aceita Output com format JSON                                   │
│   - Escreve para buffer em memória                                  │
│   - Usa config de repo.config se existir                            │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 Sistema de Tipos

#### ApiResponse<T>

```rust
#[napi(object)]
pub struct ApiResponse<T> {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Response data (present on success).
    pub data: Option<T>,
    /// Error information (present on failure).
    pub error: Option<ErrorInfo>,
}
```

#### ErrorInfo

```rust
#[napi(object)]
pub struct ErrorInfo {
    /// Error code (Node.js style: EVALIDATION, EGIT, etc.).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Additional context (field name, path, etc.).
    pub context: Option<String>,
    /// Error kind from CLI.
    pub kind: String,
}
```

#### Códigos de Erro

| Código | CliError | Descrição |
|--------|----------|-----------|
| `ECONFIG` | Configuration | Erro de configuração |
| `EVALIDATION` | Validation | Parâmetros inválidos |
| `EEXEC` | Execution | Erro de execução |
| `EGIT` | Git | Erro Git |
| `EPKG` | Package | Erro de package |
| `ENOENT` | Io (NotFound) | Ficheiro não encontrado |
| `EIO` | Io | Erro de I/O |
| `ENETWORK` | Network | Erro de rede |
| `EUSER` | User | Cancelado pelo utilizador |
| `ETIMEOUT` | (Novo) | Timeout de execução |

---

## 4. FASES DE IMPLEMENTAÇÃO

### Fase 1: Foundation (Semana 1)

**Objectivo**: Criar estrutura base do crate com tipos fundamentais.

#### Tarefas

| ID | Tarefa | Ficheiros | Estimativa |
|----|--------|-----------|------------|
| F1.1 | Criar estrutura de directórios | `crates/node/` | 0.5h |
| F1.2 | Configurar `Cargo.toml` | `crates/node/Cargo.toml` | 1h |
| F1.3 | Criar `build.rs` | `crates/node/build.rs` | 0.5h |
| F1.4 | Implementar `error.rs` | `src/error.rs` | 4h |
| F1.5 | Implementar `validation.rs` | `src/validation.rs` | 4h |
| F1.6 | Implementar `response.rs` | `src/response.rs` | 2h |
| F1.7 | Criar `types/mod.rs` e `types/common.rs` | `src/types/` | 2h |
| F1.8 | Criar `lib.rs` com exports base | `src/lib.rs` | 2h |
| F1.9 | Actualizar workspace `Cargo.toml` | `Cargo.toml` | 0.5h |
| F1.10 | Testes unitários para error e validation | `tests/` | 4h |

**Entregáveis**:
- Crate compila sem erros
- Clippy 100%
- Testes para error mapping e validação

---

### Fase 2: POC - Status Command (Semana 2)

**Objectivo**: Implementar `status` como proof-of-concept end-to-end.

#### Tarefas

| ID | Tarefa | Ficheiros | Estimativa |
|----|--------|-----------|------------|
| P2.1 | Implementar `types/status.rs` | `src/types/status.rs` | 3h |
| P2.2 | Implementar `commands/status.rs` | `src/commands/status.rs` | 6h |
| P2.3 | Actualizar `lib.rs` com export | `src/lib.rs` | 1h |
| P2.4 | Build e verificar TypeScript gerado | - | 2h |
| P2.5 | Testes Node.js para status | `packages/workspace-tools/__test__/status.test.ts` | 4h |
| P2.6 | Documentação da função | `src/commands/status.rs` | 2h |
| P2.7 | Implementar `init` (similar pattern) | `src/types/init.rs`, `src/commands/init.rs` | 6h |

**Entregáveis**:
- `status()` funcional em Node.js
- `init()` funcional em Node.js
- TypeScript types gerados correctamente
- Testes a passar

---

### Fase 3: Core Commands - Changeset (Semana 3)

**Objectivo**: Implementar todas as 7 funções de changeset.

#### Tarefas

| ID | Tarefa | Ficheiros | Estimativa |
|----|--------|-----------|------------|
| C3.1 | Implementar `types/changeset.rs` | `src/types/changeset.rs` | 4h |
| C3.2 | Implementar `changesetAdd` | `src/commands/changeset.rs` | 4h |
| C3.3 | Implementar `changesetUpdate` | `src/commands/changeset.rs` | 3h |
| C3.4 | Implementar `changesetList` | `src/commands/changeset.rs` | 3h |
| C3.5 | Implementar `changesetShow` | `src/commands/changeset.rs` | 2h |
| C3.6 | Implementar `changesetRemove` | `src/commands/changeset.rs` | 2h |
| C3.7 | Implementar `changesetHistory` | `src/commands/changeset.rs` | 3h |
| C3.8 | Implementar `changesetCheck` | `src/commands/changeset.rs` | 2h |
| C3.9 | Testes Node.js para changeset | `__test__/changeset.test.ts` | 6h |

**Entregáveis**:
- 7 funções changeset funcionais
- Testes completos

---

### Fase 4: Core Commands - Bump (Semana 4)

**Objectivo**: Implementar 3 funções de bump com suporte completo a prerelease e snapshot.

#### Tarefas

| ID | Tarefa | Ficheiros | Estimativa |
|----|--------|-----------|------------|
| B4.1 | Implementar `types/bump.rs` | `src/types/bump.rs` | 4h |
| B4.2 | Implementar `bumpPreview` | `src/commands/bump.rs` | 4h |
| B4.3 | Implementar `bumpApply` (com prerelease) | `src/commands/bump.rs` | 5h |
| B4.4 | Implementar `bumpSnapshot` | `src/commands/bump.rs` | 3h |
| B4.5 | Implementar validadores prerelease/snapshot | `src/validation.rs` | 2h |
| B4.6 | Testes Node.js para bump | `__test__/bump.test.ts` | 5h |

#### Especificação de Tipos (B4.1)

**BumpPreviewParams:**
```typescript
interface BumpPreviewParams {
  root?: string;
  configPath?: string;
  packages?: string[];
  showDiff?: boolean;
}
```

**BumpApplyParams:**
```typescript
interface BumpApplyParams {
  root?: string;
  configPath?: string;
  packages?: string[];
  
  // Git operations
  gitCommit?: boolean;
  gitTag?: boolean;
  gitPush?: boolean;
  
  // Prerelease support
  prerelease?: string;  // alpha, beta, rc, or custom tag
  
  // Changelog/Archive control
  noChangelog?: boolean;
  noArchive?: boolean;
  alwaysArchive?: boolean;
  
  // Behavior
  force?: boolean;
}
```

**BumpSnapshotParams:**
```typescript
interface BumpSnapshotParams {
  root?: string;
  configPath?: string;
  packages?: string[];
  format?: string;  // Template: {version}-snapshot.{short_commit}
}
```

#### Prerelease vs Snapshot

| Aspecto | Prerelease | Snapshot |
|---------|------------|----------|
| SemVer Compliant | ✅ Yes | ❌ No |
| Persisted | ✅ Yes (changesets archived) | ❌ No |
| Use Case | Staging/Beta releases | Testing/CI preview |
| Example | `1.3.0-beta.0` | `1.2.3-snapshot.abc123f` |
| Changelogs | Generated | Not generated |
| Git Tags | Optional | Not created |

#### Validação Adicional (B4.5)

- `prerelease_tag`: Validates tag contains only `[0-9A-Za-z-]`
- `snapshot_format`: Validates format contains valid variables

**Entregáveis**:
- 3 funções bump funcionais
- Suporte completo a prerelease
- Suporte completo a snapshot
- Validadores específicos
- Workflow de release testado

---

### Fase 5: Execute & Config (Semana 5)

**Objectivo**: Implementar execute com timeout e config commands.

#### Tarefas

| ID | Tarefa | Ficheiros | Estimativa |
|----|--------|-----------|------------|
| E5.1 | Adicionar `ExecuteConfig` ao pkg | `crates/pkg/src/config/` | 4h |
| E5.2 | Implementar `types/execute.rs` | `src/types/execute.rs` | 3h |
| E5.3 | Implementar `execute` com timeout | `src/commands/execute.rs` | 8h |
| E5.4 | Implementar `types/config.rs` | `src/types/config.rs` | 2h |
| E5.5 | Implementar `configShow` | `src/commands/config.rs` | 3h |
| E5.6 | Implementar `configValidate` | `src/commands/config.rs` | 3h |
| E5.7 | Testes para timeout e config | `__test__/` | 6h |

**Entregáveis**:
- `execute()` com timeout configurável
- Config commands funcionais
- Testes de timeout

---

### Fase 6: Remaining Commands (Semana 5-6)

**Objectivo**: Implementar Upgrade, Audit, Changes, Clone.

#### Tarefas

| ID | Tarefa | Ficheiros | Estimativa |
|----|--------|-----------|------------|
| R6.1 | Implementar `types/upgrade.rs` | `src/types/upgrade.rs` | 3h |
| R6.2 | Implementar 5 funções upgrade | `src/commands/upgrade.rs` | 8h |
| R6.3 | Implementar `types/audit.rs` | `src/types/audit.rs` | 2h |
| R6.4 | Implementar `audit` | `src/commands/audit.rs` | 4h |
| R6.5 | Implementar `types/changes.rs` | `src/types/changes.rs` | 2h |
| R6.6 | Implementar `changes` | `src/commands/changes.rs` | 3h |
| R6.7 | Implementar `types/clone.rs` | `src/types/clone.rs` | 2h |
| R6.8 | Implementar `clone` | `src/commands/clone.rs` | 4h |
| R6.9 | Testes para todos | `__test__/` | 8h |

**Entregáveis**:
- Todas as 23 funções implementadas
- Testes básicos para cada

---

### Fase 7: Polish & Release (Semana 7-8)

**Objectivo**: Qualidade, documentação, CI/CD, release.

#### Tarefas

| ID | Tarefa | Ficheiros | Estimativa |
|----|--------|-----------|------------|
| P7.1 | Clippy 100% sem warnings | - | 4h |
| P7.2 | Testes Rust 100% coverage | `tests/` | 8h |
| P7.3 | Testes Node.js >90% coverage | `__test__/` | 8h |
| P7.4 | Documentação completa (rustdoc) | Todos os ficheiros | 6h |
| P7.5 | Actualizar README do package | `packages/workspace-tools/README.md` | 4h |
| P7.6 | Actualizar `index.ts` com novos exports | `src/index.ts` | 2h |
| P7.7 | CI/CD para build multi-platform | `.github/workflows/` | 4h |
| P7.8 | Performance benchmarks | `benchmark/` | 4h |
| P7.9 | Release npm | - | 2h |

**Entregáveis**:
- Package pronto para produção
- Documentação completa
- CI/CD funcional

---

## 5. ESPECIFICAÇÕES TÉCNICAS DETALHADAS

### 5.1 Cargo.toml do Crate Node

```toml
[package]
name = "sublime_node_tools"
version = "0.0.1"
edition = "2024"
authors = ["WebSublime"]
license = "MIT"
repository = "https://github.com/websublime/workspace-tools"
description = "Node.js bindings for Sublime Workspace CLI Tools"
readme = "README.md"

[lib]
crate-type = ["cdylib"]

[dependencies]
# NAPI
napi = { version = "3.0.0", features = ["async", "tokio_rt", "napi9"] }
napi-derive = "3.0.0"

# Workspace crates
sublime_cli_tools = { workspace = true }
sublime_git_tools = { workspace = true }
sublime_pkg_tools = { workspace = true }
sublime_standard_tools = { workspace = true }

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }

# Async runtime
tokio = { workspace = true, features = ["full", "time"] }

# Tracing
tracing = { workspace = true }

[build-dependencies]
napi-build = "3.0.0"

[lints]
workspace = true

# Clippy overrides específicos para NAPI
[lints.clippy]
# NAPI requer Some em certos contextos
needless_option_as_deref = "allow"
```

### 5.2 build.rs

```rust
//! Build script for napi-rs bindings.
//!
//! # What
//!
//! This build script configures napi-rs to generate Node.js bindings.
//!
//! # How
//!
//! Uses `napi_build::setup()` to configure the build environment.
//!
//! # Why
//!
//! Required by napi-rs to properly link and generate bindings.

fn main() {
    napi_build::setup();
}
```

### 5.3 lib.rs Entry Point

```rust
//! Node.js bindings for Sublime Workspace CLI Tools.
//!
//! # What
//!
//! This crate provides Node.js bindings via napi-rs for the workspace-tools CLI,
//! exposing 23 execute functions as native JavaScript/TypeScript functions.
//!
//! # How
//!
//! Uses `#[napi]` macros to:
//! - Define async functions callable from Node.js
//! - Convert Rust types to JavaScript objects via `#[napi(object)]`
//! - Generate TypeScript definitions automatically
//!
//! # Why
//!
//! Enables Node.js/TypeScript applications to use workspace-tools functionality
//! programmatically without spawning CLI processes, with full type safety.
//!
//! # Examples
//!
//! ```typescript
//! import { status, changesetAdd, bumpPreview } from '@websublime/workspace-tools';
//!
//! const result = await status({ root: '.' });
//! if (result.success) {
//!   console.log(result.data.packages);
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

#[macro_use]
extern crate napi_derive;

mod error;
mod response;
mod validation;

pub(crate) mod commands;
pub(crate) mod types;

// Re-export all NAPI functions
pub use commands::status::status;
pub use commands::init::init;
// ... other exports
```

### 5.4 Estrutura de types/status.rs

```rust
//! Status command types for NAPI bindings.
//!
//! # What
//!
//! Defines the parameter and response types for the `status` command.
//!
//! # How
//!
//! Uses `#[napi(object)]` to generate JavaScript-compatible types with
//! automatic TypeScript definitions.
//!
//! # Why
//!
//! Provides type-safe interfaces between JavaScript and Rust.

use napi_derive::napi;

/// Parameters for the status command.
#[napi(object)]
pub struct StatusParams {
    /// Workspace root directory path.
    /// If not provided, defaults to current working directory.
    pub root: Option<String>,
    /// Optional custom config file path.
    pub config_path: Option<String>,
}

/// Repository type information.
#[napi(object)]
pub struct RepositoryInfo {
    /// Repository kind: "simple" or "monorepo".
    pub kind: String,
    /// Monorepo type if applicable (npm, yarn, pnpm, bun, deno, custom).
    pub monorepo_type: Option<String>,
}

/// Package manager information.
#[napi(object)]
pub struct PackageManagerInfo {
    /// Package manager name (npm, yarn, pnpm, bun, jsr).
    pub name: String,
    /// Lock file name.
    pub lock_file: String,
}

/// Git branch information.
#[napi(object)]
pub struct BranchInfo {
    /// Branch name.
    pub name: String,
}

/// Changeset information.
#[napi(object)]
pub struct ChangesetInfo {
    /// Changeset ID (branch name).
    pub id: String,
}

/// Package information.
#[napi(object)]
pub struct PackageInfo {
    /// Package name (may include scope).
    pub name: String,
    /// Package version.
    pub version: String,
    /// Package path relative to workspace root.
    pub path: String,
}

/// Status command response data.
#[napi(object)]
pub struct StatusData {
    /// Repository type information.
    pub repository: RepositoryInfo,
    /// Package manager information.
    pub package_manager: PackageManagerInfo,
    /// Current Git branch (if available).
    pub branch: Option<BranchInfo>,
    /// List of pending changesets.
    pub changesets: Vec<ChangesetInfo>,
    /// List of workspace packages.
    pub packages: Vec<PackageInfo>,
}
```

### 5.5 Estrutura de commands/status.rs

```rust
//! Status command implementation for NAPI bindings.
//!
//! # What
//!
//! Implements the `status` function that retrieves workspace information.
//!
//! # How
//!
//! 1. Validates input parameters
//! 2. Creates an in-memory buffer for CLI output
//! 3. Calls `execute_status` from CLI with JSON format
//! 4. Parses JSON response
//! 5. Converts to NAPI types
//! 6. Returns `ApiResponse<StatusData>`
//!
//! # Why
//!
//! Provides workspace status information to Node.js applications.

use crate::error::cli_error_to_info;
use crate::response::ApiResponse;
use crate::types::status::{
    BranchInfo, ChangesetInfo, PackageInfo, PackageManagerInfo, RepositoryInfo, StatusData,
    StatusParams,
};
use crate::validation::validators;
use napi_derive::napi;
use std::io::Cursor;
use std::path::Path;
use sublime_cli_tools::cli::commands::StatusArgs;
use sublime_cli_tools::commands::status::execute_status;
use sublime_cli_tools::output::{Output, OutputFormat};

/// Get workspace status information.
///
/// Returns comprehensive information about the workspace including repository type,
/// package manager, Git branch, pending changesets, and all workspace packages.
///
/// @param params - Status parameters (root, configPath)
/// @returns Promise<ApiResponse<StatusData>> - Status information or error
///
/// @example
/// ```typescript
/// const result = await status({ root: '/path/to/project' });
/// if (result.success) {
///   console.log(`Found ${result.data.packages.length} packages`);
///   console.log(`Package manager: ${result.data.packageManager.name}`);
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
#[napi]
pub async fn status(params: StatusParams) -> ApiResponse<StatusData> {
    // Validate parameters
    let root_path = match &params.root {
        Some(path) => {
            if let Err(e) = validators::path_exists(path) {
                return ApiResponse::failure(e.into());
            }
            Path::new(path).to_path_buf()
        }
        None => match std::env::current_dir() {
            Ok(p) => p,
            Err(e) => {
                return ApiResponse::failure_from_io(e);
            }
        },
    };

    let config_path = params.config_path.as_ref().map(Path::new);

    // Create in-memory buffer for output
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, true);

    // Create args
    let args = StatusArgs {};

    // Execute command
    match execute_status(&args, &output, &root_path, config_path).await {
        Ok(()) => {
            // Parse JSON from buffer
            let json_bytes = output.into_inner().into_inner();
            match parse_status_response(&json_bytes) {
                Ok(data) => ApiResponse::success(data),
                Err(e) => ApiResponse::failure(e),
            }
        }
        Err(cli_error) => ApiResponse::failure(cli_error_to_info(&cli_error)),
    }
}

/// Parse status JSON response into StatusData.
fn parse_status_response(json_bytes: &[u8]) -> Result<StatusData, crate::error::ErrorInfo> {
    // Parse and convert JSON to StatusData
    // Implementation details...
    todo!()
}
```

### 5.6 Configuração ExecuteConfig

Adicionar ao `crates/pkg/src/config/`:

```rust
//! Execute command configuration.
//!
//! # What
//!
//! Defines configuration options for the `execute` command including
//! timeout settings and parallelism limits.
//!
//! # How
//!
//! Loaded from `repo.config` under the `[execute]` section.
//!
//! # Why
//!
//! Provides sensible defaults while allowing project-specific overrides.

use serde::{Deserialize, Serialize};

/// Execute command configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecuteConfig {
    /// Default timeout for execute commands in seconds.
    /// 0 means no timeout.
    /// Default: 300 (5 minutes)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Timeout per individual package in seconds.
    /// Default: 60 (1 minute)
    #[serde(default = "default_per_package_timeout")]
    pub per_package_timeout_secs: u64,

    /// Maximum number of parallel executions.
    /// Default: 8
    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
}

impl Default for ExecuteConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            per_package_timeout_secs: default_per_package_timeout(),
            max_parallel: default_max_parallel(),
        }
    }
}

fn default_timeout() -> u64 {
    300
}

fn default_per_package_timeout() -> u64 {
    60
}

fn default_max_parallel() -> usize {
    8
}
```

### 5.7 Execute Command com Timeout

```rust
//! Execute command implementation for NAPI bindings.
//!
//! # What
//!
//! Implements the `execute` function that runs commands across workspace packages.
//!
//! # How
//!
//! 1. Validates input parameters (including mutual exclusions)
//! 2. Loads timeout config from repo.config with optional parameter override
//! 3. Calls `execute_execute` from CLI
//! 4. Applies timeout via tokio::time::timeout
//! 5. Returns `ApiResponse<ExecuteData>`
//!
//! # Why
//!
//! Enables running commands across packages programmatically with timeout protection.

use crate::error::{cli_error_to_info, ErrorInfo};
use crate::response::ApiResponse;
use crate::types::execute::{ExecuteData, ExecuteParams, ExecuteSummary, PackageExecutionResult};
use crate::validation::validators;
use napi_derive::napi;
use std::path::Path;
use std::time::Duration;
use sublime_cli_tools::cli::commands::ExecuteArgs;
use sublime_cli_tools::commands::execute::execute_execute;
use sublime_cli_tools::output::{Output, OutputFormat};
use tokio::time::timeout;

/// Execute a command across workspace packages.
///
/// Runs the specified command on workspace packages with optional filtering,
/// parallel execution, and timeout protection.
///
/// @param params - Execute parameters
/// @returns Promise<ApiResponse<ExecuteData>> - Execution results or error
///
/// @example
/// ```typescript
/// // Run tests on affected packages
/// const result = await execute({
///   root: '.',
///   cmd: 'npm:test',
///   affected: true,
///   branch: 'main',
///   parallel: true,
///   timeoutSecs: 600,
/// });
///
/// if (result.success) {
///   console.log(`${result.data.summary.succeeded}/${result.data.summary.total} passed`);
/// }
/// ```
#[napi]
pub async fn execute(params: ExecuteParams) -> ApiResponse<ExecuteData> {
    // Validate parameters
    if let Err(e) = validate_execute_params(&params) {
        return ApiResponse::failure(e);
    }

    // Load config for timeout defaults
    let (overall_timeout, _per_package_timeout) = resolve_timeouts(&params);

    // Create args
    let args = ExecuteArgs {
        cmd: params.cmd.clone(),
        filter_package: params.filter_package.clone(),
        affected: params.affected.unwrap_or(false),
        since: params.since.clone(),
        until: params.until.clone(),
        branch: params.branch.clone(),
        parallel: params.parallel.unwrap_or(false),
        args: params.args.clone().unwrap_or_default(),
    };

    // Execute with timeout
    let root_path = resolve_root(&params.root);
    let execute_future = execute_command(&args, &root_path);

    if overall_timeout > 0 {
        match timeout(Duration::from_secs(overall_timeout), execute_future).await {
            Ok(result) => result,
            Err(_) => ApiResponse::failure(ErrorInfo {
                code: "ETIMEOUT".to_string(),
                message: format!("Execution timed out after {overall_timeout} seconds"),
                context: Some("execute".to_string()),
                kind: "Timeout".to_string(),
            }),
        }
    } else {
        execute_future.await
    }
}

fn validate_execute_params(params: &ExecuteParams) -> Result<(), ErrorInfo> {
    // Validate cmd is not empty
    validators::not_empty("cmd", &params.cmd)?;

    // Validate mutual exclusion: filterPackage and affected
    if params.filter_package.is_some() && params.affected.unwrap_or(false) {
        return Err(ErrorInfo {
            code: "EVALIDATION".to_string(),
            message: "Cannot use both filterPackage and affected".to_string(),
            context: Some("filterPackage, affected".to_string()),
            kind: "Validation".to_string(),
        });
    }

    // Validate timeout ranges
    if let Some(t) = params.timeout_secs {
        validators::timeout("timeoutSecs", t, 0, 86400)?; // max 24h
    }
    if let Some(t) = params.per_package_timeout_secs {
        validators::timeout("perPackageTimeoutSecs", t, 0, 3600)?; // max 1h
    }

    Ok(())
}

fn resolve_timeouts(params: &ExecuteParams) -> (u64, u64) {
    // TODO: Load from config, apply overrides
    let overall = params.timeout_secs.unwrap_or(300);
    let per_package = params.per_package_timeout_secs.unwrap_or(60);
    (overall, per_package)
}

fn resolve_root(root: &Option<String>) -> std::path::PathBuf {
    match root {
        Some(p) => Path::new(p).to_path_buf(),
        None => std::env::current_dir().unwrap_or_default(),
    }
}

async fn execute_command(args: &ExecuteArgs, root: &Path) -> ApiResponse<ExecuteData> {
    // Implementation details...
    todo!()
}
```

---

## 6. TAREFAS POR SPRINT

### Sprint 1: Foundation (Dias 1-5)

| Dia | Tarefas | Output |
|-----|---------|--------|
| 1 | F1.1-F1.3: Criar estrutura, Cargo.toml, build.rs | Crate compila |
| 2 | F1.4: Implementar error.rs completo | ErrorInfo, códigos, conversão |
| 3 | F1.5: Implementar validation.rs | Validadores funcionais |
| 4 | F1.6-F1.8: response.rs, types/common.rs, lib.rs | ApiResponse funcional |
| 5 | F1.9-F1.10: Actualizar workspace, testes | Clippy 100%, testes a passar |

### Sprint 2: POC (Dias 6-10)

| Dia | Tarefas | Output |
|-----|---------|--------|
| 6 | P2.1-P2.2: types/status.rs, commands/status.rs | status() compila |
| 7 | P2.3-P2.4: Exports, verificar TypeScript | status() callable de JS |
| 8 | P2.5: Testes Node.js para status | Testes a passar |
| 9 | P2.7: Implementar init | init() funcional |
| 10 | Buffer, fix issues, documentação | POC estável |

### Sprint 3: Changeset (Dias 11-15)

| Dia | Tarefas | Output |
|-----|---------|--------|
| 11 | C3.1-C3.2: types/changeset.rs, changesetAdd | Add funcional |
| 12 | C3.3-C3.4: changesetUpdate, changesetList | Update e List funcionais |
| 13 | C3.5-C3.6: changesetShow, changesetRemove | Show e Remove funcionais |
| 14 | C3.7-C3.8: changesetHistory, changesetCheck | Todas changeset funcionais |
| 15 | C3.9: Testes Node.js | 7 funções testadas |

### Sprint 4: Bump (Dias 16-20)

| Dia | Tarefas | Output |
|-----|---------|--------|
| 16 | B4.1-B4.2: types/bump.rs, bumpPreview | Preview funcional |
| 17 | B4.3: bumpApply | Apply funcional |
| 18 | B4.4: bumpSnapshot | Snapshot funcional |
| 19 | B4.5: Testes Node.js | 3 funções testadas |
| 20 | Buffer, integration tests | Bump estável |

### Sprint 5: Execute & Config (Dias 21-25)

| Dia | Tarefas | Output |
|-----|---------|--------|
| 21 | E5.1: ExecuteConfig no pkg | Config loading funcional |
| 22 | E5.2-E5.3: types/execute.rs, execute com timeout | execute() básico |
| 23 | E5.3: Finalizar timeout handling | Timeout funcional |
| 24 | E5.4-E5.6: Config commands | configShow, configValidate |
| 25 | E5.7: Testes | Execute e Config testados |

### Sprint 6: Remaining (Dias 26-32)

| Dia | Tarefas | Output |
|-----|---------|--------|
| 26 | R6.1-R6.2: Upgrade (2/5) | upgradeCheck, upgradeApply |
| 27 | R6.2: Upgrade (3/5) | backup* funções |
| 28 | R6.3-R6.4: Audit | audit() funcional |
| 29 | R6.5-R6.6: Changes | changes() funcional |
| 30 | R6.7-R6.8: Clone | clone() funcional |
| 31-32 | R6.9: Testes | Todas funções testadas |

### Sprint 7-8: Polish (Dias 33-40)

| Dia | Tarefas | Output |
|-----|---------|--------|
| 33-34 | P7.1-P7.2: Clippy 100%, Rust tests | Qualidade máxima |
| 35-36 | P7.3: Node.js tests >90% | Coverage target |
| 37 | P7.4-P7.5: Documentação | Docs completos |
| 38 | P7.6-P7.7: Exports, CI/CD | Pipeline funcional |
| 39 | P7.8: Benchmarks | Performance validada |
| 40 | P7.9: Release | v1.1.0 publicado |

---

## 7. CRITÉRIOS DE ACEITAÇÃO

### 7.1 Por Função

Cada função NAPI deve:

- [ ] Compilar sem warnings (Clippy 100%)
- [ ] Ter documentação completa (rustdoc)
- [ ] Validar parâmetros antes de invocar CLI
- [ ] Retornar `ApiResponse<T>` com tipos correctos
- [ ] Gerar TypeScript definitions corretos
- [ ] Ter pelo menos 3 testes Node.js (success, validation error, CLI error)
- [ ] Ter exemplo de uso na documentação

### 7.2 Por Fase

| Fase | Critérios |
|------|-----------|
| Foundation | Crate compila, Clippy 100%, error mapping testado |
| POC | status() e init() chamáveis de Node.js com tipos correctos |
| Changeset | 7 funções funcionais, workflow de changeset testado |
| Bump | Release workflow end-to-end funcional |
| Execute | Timeout funciona, parallel execution testada |
| Remaining | Todas 23 funções implementadas e testadas |
| Polish | 100% Rust coverage, >90% Node.js coverage, docs completos |

### 7.3 Global

- [ ] Clippy 100% sem warnings em todo o crate
- [ ] Todos os 10 error codes mapeados e testados
- [ ] TypeScript definitions gerados sem erros
- [ ] Builds em macOS, Linux, Windows (x64 e ARM64)
- [ ] Performance: <50ms overhead vs CLI directo
- [ ] README actualizado com exemplos

---

## 8. RISCOS E MITIGAÇÕES

### 8.1 Riscos Técnicos

| Risco | Probabilidade | Impacto | Mitigação |
|-------|---------------|---------|-----------|
| Conversão JSON→NAPI complexa | Média | Médio | Módulo dedicado para parsing, testes extensivos |
| CLI breaking changes | Baixa | Alto | Mesmo repo = mudanças atómicas, testes detectam |
| Platform-specific issues | Média | Médio | CI testa todas plataformas, napi-rs é maduro |
| Timeout handling edge cases | Média | Médio | Testes específicos para timeouts e cancellation |
| Memory leaks em long-running ops | Baixa | Alto | Profiling, proper cleanup handlers |

### 8.2 Riscos de Timeline

| Risco | Probabilidade | Impacto | Mitigação |
|-------|---------------|---------|-----------|
| Subestimação de complexidade | Média | Médio | Buffer de 1 semana, POC valida estimativas |
| Integração com CLI mais difícil | Baixa | Médio | POC early, feedback rápido |
| Testes Node.js demorados | Média | Baixo | Paralelização, fixtures reutilizáveis |

### 8.3 Decisões de Fallback

| Se... | Então... |
|-------|----------|
| Conversão de tipos muito complexa | Simplificar para JSON string temporariamente |
| Timeout difícil de implementar | Lançar sem timeout, adicionar depois |
| Uma função específica muito problemática | Marcar como experimental, documentar limitações |

---

## 9. CHECKLIST DE QUALIDADE

### 9.1 Checklist de Código (Por Ficheiro)

```
[ ] Clippy sem warnings
[ ] Documentação de módulo (//! What, How, Why)
[ ] Documentação de todos os structs públicos
[ ] Documentação de todos os métodos/funções
[ ] Exemplos em doc comments
[ ] Sem #[allow(clippy::...)] desnecessários
[ ] Imports organizados
[ ] Nomes descritivos
```

### 9.2 Checklist de Testes (Por Função)

```
[ ] Teste de success case
[ ] Teste de validation error (parâmetros inválidos)
[ ] Teste de CLI error (erro interno)
[ ] Teste de edge cases (paths especiais, caracteres unicode)
[ ] Teste de performance (tempo de resposta)
```

### 9.3 Checklist de Release

```
[ ] Versão bumped em Cargo.toml e package.json
[ ] CHANGELOG actualizado
[ ] TypeScript types verificados manualmente
[ ] Build em todas plataformas (CI)
[ ] Testes a passar em todas plataformas
[ ] README com exemplos actualizados
[ ] npm publish com --dry-run primeiro
[ ] Git tag criado
[ ] GitHub release criado
```

### 9.4 Comandos de Verificação

```bash
# Clippy (crate node)
cargo clippy --all-targets -p sublime_node_tools -- -D warnings

# Testes Rust
cargo test -p sublime_node_tools

# Coverage Rust
cargo tarpaulin --out Html -p sublime_node_tools

# Build bindings
pnpm build-binding:release

# Verificar TypeScript
npx tsc --noEmit

# Testes Node.js
pnpm test

# Coverage Node.js
pnpm test -- --coverage
```

---

## APÊNDICES

### A. Mapeamento Completo de Funções

| # | CLI Function | NAPI Function | Args Struct | Response Type |
|---|--------------|---------------|-------------|---------------|
| 1 | `execute_status` | `status` | `StatusParams` | `StatusData` |
| 2 | `execute_init` | `init` | `InitParams` | `InitData` |
| 3 | `execute_show` | `configShow` | `ConfigShowParams` | `ConfigData` |
| 4 | `execute_validate` | `configValidate` | `ConfigValidateParams` | `ConfigValidationData` |
| 5 | `execute_add` | `changesetAdd` | `ChangesetAddParams` | `ChangesetData` |
| 6 | `execute_update` | `changesetUpdate` | `ChangesetUpdateParams` | `ChangesetData` |
| 7 | `execute_list` | `changesetList` | `ChangesetListParams` | `ChangesetListData` |
| 8 | `execute_show` | `changesetShow` | `ChangesetShowParams` | `ChangesetData` |
| 9 | `execute_remove` | `changesetRemove` | `ChangesetRemoveParams` | `void` |
| 10 | `execute_history` | `changesetHistory` | `ChangesetHistoryParams` | `ChangesetHistoryData` |
| 11 | `execute_check` | `changesetCheck` | `ChangesetCheckParams` | `ChangesetCheckData` |
| 12 | `execute_bump_preview` | `bumpPreview` | `BumpPreviewParams` | `BumpPreviewData` |
| 13 | `execute_bump_apply` | `bumpApply` | `BumpApplyParams` | `BumpApplyData` |
| 14 | `execute_bump_snapshot` | `bumpSnapshot` | `BumpSnapshotParams` | `BumpSnapshotData` |
| 15 | `execute_upgrade_check` | `upgradeCheck` | `UpgradeCheckParams` | `UpgradeCheckData` |
| 16 | `execute_upgrade_apply` | `upgradeApply` | `UpgradeApplyParams` | `UpgradeApplyData` |
| 17 | `execute_backup_list` | `backupList` | `BackupListParams` | `BackupListData` |
| 18 | `execute_backup_restore` | `backupRestore` | `BackupRestoreParams` | `void` |
| 19 | `execute_backup_clean` | `backupClean` | `BackupCleanParams` | `BackupCleanData` |
| 20 | `execute_audit` | `audit` | `AuditParams` | `AuditData` |
| 21 | `execute_changes` | `changes` | `ChangesParams` | `ChangesData` |
| 22 | `execute_clone` | `clone` | `CloneParams` | `CloneData` |
| 23 | `execute_execute` | `execute` | `ExecuteParams` | `ExecuteData` |

### B. Parâmetros Fixados na API

| Parâmetro CLI | Valor Fixo | Justificação |
|---------------|------------|--------------|
| `--non-interactive` | `true` | APIs não devem ter prompts |
| `--format` | N/A (objectos) | Output estruturado nativo |
| `--no-color` | `true` | Sem terminal |

### C. repo.config Example

```toml
[execute]
# Default timeout for execute commands (seconds)
# 0 = no timeout
timeout_secs = 300

# Timeout per individual package (seconds)
per_package_timeout_secs = 60

# Maximum concurrent parallel executions
max_parallel = 8
```

### D. Exemplo de TypeScript Usage

```typescript
import {
  status,
  changesetAdd,
  changesetList,
  bumpPreview,
  bumpApply,
  execute,
  type ApiResponse,
  type StatusData,
  type ChangesetData,
} from '@websublime/workspace-tools';

async function releaseWorkflow() {
  const root = process.cwd();

  // 1. Check workspace status
  const statusResult = await status({ root });
  if (!statusResult.success) {
    throw new Error(`Status failed: ${statusResult.error?.message}`);
  }

  console.log(`Found ${statusResult.data?.packages.length} packages`);

  // 2. List pending changesets
  const changesets = await changesetList({ root });
  if (!changesets.success || changesets.data?.changesets.length === 0) {
    console.log('No pending changesets');
    return;
  }

  // 3. Preview bump
  const preview = await bumpPreview({ root, showDiff: true });
  if (!preview.success) {
    throw new Error(`Preview failed: ${preview.error?.message}`);
  }

  console.log('Changes to apply:');
  preview.data?.changes.forEach((c) => {
    console.log(`  ${c.package}: ${c.oldVersion} → ${c.newVersion}`);
  });

  // 4. Apply bump
  const apply = await bumpApply({
    root,
    execute: true,
    gitCommit: true,
    gitTag: true,
  });

  if (!apply.success) {
    throw new Error(`Apply failed: ${apply.error?.message}`);
  }

  console.log('Release complete!');
}

async function runTestsOnAffected() {
  const result = await execute({
    root: '.',
    cmd: 'npm:test',
    affected: true,
    branch: 'main',
    parallel: true,
    timeoutSecs: 600,
    perPackageTimeoutSecs: 120,
  });

  if (!result.success) {
    if (result.error?.code === 'ETIMEOUT') {
      console.error('Tests timed out!');
    } else {
      console.error(`Error: ${result.error?.message}`);
    }
    process.exit(1);
  }

  const { summary } = result.data!;
  console.log(`Tests: ${summary.succeeded}/${summary.total} passed`);

  if (summary.failed > 0) {
    process.exit(1);
  }
}
```

---

**Documento mantido por**: Equipa de Desenvolvimento  
**Última actualização**: 2025-01-20  
**Próxima revisão**: Após conclusão da Fase 1