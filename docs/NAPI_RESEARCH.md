# RELATÓRIO COMPLETO: Node.js Bindings com napi-rs para Sublime Workspace CLI Tools

**Projeto**: workspace-node-tools  
**Objetivo**: Criar bindings Node.js para expor 20 funções `execute_*` do CLI  
**Tecnologia**: napi-rs (versão 3.x)  
**Data**: 2025-01-18  
**Status**: Research & Planning

---

## ÍNDICE

1. [Executive Summary](#1-executive-summary)
2. [Estado Atual do Projeto](#2-estado-atual-do-projeto)
3. [Inventário Completo de Funções Execute](#3-inventário-completo-de-funções-execute)
4. [Análise Detalhada do Código Existente](#4-análise-detalhada-do-código-existente)
5. [Arquitetura napi-rs: Fundamentos](#5-arquitetura-napi-rs-fundamentos)
6. [Estratégia de Implementação](#6-estratégia-de-implementação)
7. [Arquitetura Detalhada Proposta](#7-arquitetura-detalhada-proposta)
8. [Especificação de Cada Função](#8-especificação-de-cada-função)
9. [Padrões de Código e Implementação](#9-padrões-de-código-e-implementação)
10. [Error Handling Completo](#10-error-handling-completo)
11. [Testing Strategy](#11-testing-strategy)
12. [Build, Distribution & CI/CD](#12-build-distribution--cicd)
13. [Exemplos de Uso Completos](#13-exemplos-de-uso-completos)
14. [Performance & Optimization](#14-performance--optimization)
15. [Roadmap de Implementação](#15-roadmap-de-implementação)
16. [Riscos, Mitigações e Decisões](#16-riscos-mitigações-e-decisões)
17. [Apêndices](#17-apêndices)

---

## 1. EXECUTIVE SUMMARY

### 1.1 Contexto

O projeto workspace-node-tools é um conjunto de ferramentas Rust para gerenciamento de monorepos Node.js. Atualmente possui:
- **CLI completo** (`sublime_cli_tools`) com 27 funções `execute_*`
- **3 crates base**: `sublime_git_tools`, `sublime_pkg_tools`, `sublime_standard_tools`
- **1 package Node.js existente**: `@websublime/workspace-tools` (v1.0.2) com bindings básicos para git e standard tools

### 1.2 Objetivo

Expandir os bindings Node.js para incluir **20 funções execute do CLI**, permitindo que aplicações Node.js/TypeScript usem toda a funcionalidade de gerenciamento de workspace programaticamente, sem depender do CLI de terminal.

### 1.3 Escopo

**Funções a implementar (20):**
- **Init** (1): `init`
- **Config** (2): `configShow`, `configValidate`
- **Changeset** (7): `changesetAdd`, `changesetUpdate`, `changesetList`, `changesetShow`, `changesetRemove`, `changesetHistory`, `changesetCheck`
- **Bump** (3): `bumpPreview`, `bumpApply`, `bumpSnapshot`
- **Upgrade** (5): `upgradeCheck`, `upgradeApply`, `backupList`, `backupRestore`, `backupClean`
- **Audit** (1): `audit`
- **Changes** (1): `changes`
- **Clone** (1): `clone`

### 1.4 Benefícios

✅ **Reuso 100% da lógica**: Zero duplicação, chamamos execute functions diretamente  
✅ **Type-safe**: TypeScript definitions geradas automaticamente  
✅ **Cross-platform**: macOS, Linux, Windows (ARM64 + x64)  
✅ **Performance**: Nativo, sem overhead de spawnar processos CLI  
✅ **Developer-friendly**: JSON output, error handling robusto  
✅ **Maintainable**: Mudanças no CLI refletem automaticamente nos bindings  

### 1.5 Decisão Arquitetural Principal

**Abordagem escolhida: JSON String Output**

Cada função napi retorna `Promise<String>` contendo JSON, em vez de objetos JavaScript complexos.

**Rationale:**
- ✅ Simples de implementar (sem conversão complexa Rust ↔ JS)
- ✅ Universal e testável
- ✅ Evita problemas de ownership e lifetime
- ✅ Permite capturar output existente do CLI
- ✅ Performance excelente (uma serialização JSON apenas)

---

## 2. ESTADO ATUAL DO PROJETO

### 2.1 Estrutura de Diretórios

```
workspace-node-tools/
├── crates/
│   ├── cli/              # CLI principal com execute functions
│   ├── git/              # Git tools (RepoError, etc)
│   ├── pkg/              # Package tools (changeset, bump, etc)
│   └── standard/         # Standard tools (workspace detection, etc)
├── packages/
│   └── workspace-tools/  # Package Node.js EXISTENTE
│       ├── package.json  # @websublime/workspace-tools v1.0.2
│       ├── src/
│       │   ├── binding.d.ts   # TypeScript defs GERADAS
│       │   ├── binding.js     # JavaScript wrapper GERADO
│       │   ├── index.ts       # Entry point
│       │   └── workspace-tools.darwin-arm64.node  # Native binary
│       └── npm/          # Platform-specific packages
├── Cargo.toml            # Workspace root
└── package.json          # Workspace root
```

### 2.2 Package Existente: @websublime/workspace-tools

**Versão atual**: 1.0.2  
**Exports atuais**:
- `MonorepoProject` (classe)
- `MonorepoRepository` (classe)
- `GitCommit`, `GitTag`, `GitChangedFile`, `GitFileStatus`
- `getVersion()` function

**Observação importante**: Já existe infraestrutura napi-rs funcionando! Precisamos **expandir** este package, não criar um novo.

### 2.3 Configuração Existente

#### package.json (atual)
```json
{
  "name": "@websublime/workspace-tools",
  "version": "1.0.2",
  "napi": {
    "binaryName": "workspace-tools",
    "packageName": "@websublime/workspace-tools",
    "targets": [
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "x86_64-pc-windows-msvc",
      "x86_64-unknown-linux-gnu",
      "x86_64-unknown-linux-musl",
      "aarch64-unknown-linux-gnu",
      "aarch64-unknown-linux-musl",
      "aarch64-pc-windows-msvc"
    ]
  }
}
```

### 2.4 Workspace Cargo.toml

```toml
[workspace]
members = ["crates/*"]  # cli, git, pkg, standard

[workspace.dependencies]
sublime_cli_tools = { version = "0.0.13", path = "crates/cli" }
sublime_git_tools = { version = "0.0.12", path = "crates/git" }
sublime_pkg_tools = { version = "0.0.12", path = "crates/pkg" }
sublime_standard_tools = { version = "0.0.11", path = "crates/standard" }
```

**Decisão**: Precisamos adicionar um novo crate `crates/node/` ao workspace.

---

## 3. INVENTÁRIO COMPLETO DE FUNÇÕES EXECUTE

### 3.1 Tabela Master de Funções

| # | Categoria | Função Rust | Função JS | Arquivo | Linha | Args Struct |
|---|-----------|-------------|-----------|---------|-------|-------------|
| 1 | Init | `execute_init` | `init` | `commands/init.rs` | 88 | `InitArgs` |
| 2 | Config | `execute_show` | `configShow` | `commands/config.rs` | 94 | `ConfigShowArgs` |
| 3 | Config | `execute_validate` | `configValidate` | `commands/config.rs` | 210 | `ConfigValidateArgs` |
| 4 | Changeset | `execute_add` | `changesetAdd` | `commands/changeset/add.rs` | 147 | `ChangesetCreateArgs` |
| 5 | Changeset | `execute_update` | `changesetUpdate` | `commands/changeset/update.rs` | 201 | `ChangesetUpdateArgs` |
| 6 | Changeset | `execute_list` | `changesetList` | `commands/changeset/list.rs` | 164 | `ChangesetListArgs` |
| 7 | Changeset | `execute_show` | `changesetShow` | `commands/changeset/show.rs` | 155 | `ChangesetShowArgs` |
| 8 | Changeset | `execute_remove` | `changesetRemove` | `commands/changeset/remove.rs` | 175 | `ChangesetDeleteArgs` |
| 9 | Changeset | `execute_history` | `changesetHistory` | `commands/changeset/history.rs` | 140 | `ChangesetHistoryArgs` |
| 10 | Changeset | `execute_check` | `changesetCheck` | `commands/changeset/check.rs` | 136 | `ChangesetCheckArgs` |
| 11 | Bump | `execute_bump_preview` | `bumpPreview` | `commands/bump/preview.rs` | 138 | `BumpArgs` |
| 12 | Bump | `execute_bump_apply` | `bumpApply` | `commands/bump/execute.rs` | 189 | `BumpArgs` |
| 13 | Bump | `execute_bump_snapshot` | `bumpSnapshot` | `commands/bump/snapshot.rs` | 417 | `BumpArgs` |
| 14 | Upgrade | `execute_upgrade_check` | `upgradeCheck` | `commands/upgrade/check.rs` | 116 | `UpgradeCheckArgs` |
| 15 | Upgrade | `execute_upgrade_apply` | `upgradeApply` | `commands/upgrade/apply.rs` | 128 | `UpgradeApplyArgs` |
| 16 | Upgrade | `execute_backup_list` | `backupList` | `commands/upgrade/rollback.rs` | 228 | `UpgradeBackupListArgs` |
| 17 | Upgrade | `execute_backup_restore` | `backupRestore` | `commands/upgrade/rollback.rs` | 320 | `UpgradeBackupRestoreArgs` |
| 18 | Upgrade | `execute_backup_clean` | `backupClean` | `commands/upgrade/rollback.rs` | 426 | `UpgradeBackupCleanArgs` |
| 19 | Audit | `execute_audit` | `audit` | `commands/audit/comprehensive.rs` | 467 | `AuditArgs` |
| 20 | Changes | `execute_changes` | `changes` | `commands/changes.rs` | 146 | `ChangesArgs` |
| 21 | Clone | `execute_clone` | `clone` | `commands/clone.rs` | 798 | `CloneArgs` |

**Total**: 21 funções

### 3.2 Padrões de Assinatura Identificados

#### Padrão A: Legacy (Init, Config, Clone)
```rust
async fn execute_*(
    args: &ArgsStruct,
    root: &Path,
    format: OutputFormat
) -> Result<()>
```

**Características:**
- Usa `OutputFormat` enum diretamente
- Não aceita `config_path` separado
- Path sempre `&Path` (não opcional)

#### Padrão B: Moderno (Changeset, Bump, Upgrade, Audit, Changes)
```rust
async fn execute_*(
    args: &ArgsStruct,
    output: &Output,
    root: &Path (ou Option<&Path>),
    config_path: Option<&Path>
) -> Result<()>
```

**Características:**
- Usa struct `Output` (wraps writer + format + no_color)
- Config path separado e opcional
- Root pode ser opcional

---

## 4. ANÁLISE DETALHADA DO CÓDIGO EXISTENTE

### 4.1 CliError: Sistema de Erros

**Arquivo**: `crates/cli/src/error/cli_error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    Configuration(String),  // Exit code: 78
    Validation(String),     // Exit code: 65
    Execution(String),      // Exit code: 70
    Git(String),            // Exit code: 70
    Package(String),        // Exit code: 65
    Io(String),             // Exit code: 74
    Network(String),        // Exit code: 69
    User(String),           // Exit code: 64
}
```

**Métodos úteis:**
- `exit_code() -> i32`: Retorna exit code sysexits
- `kind() -> &'static str`: Retorna categoria como string
- `user_message() -> String`: Mensagem user-friendly
- `as_ref() -> &str`: Retorna "CliError::Category"

**Conversões automáticas FROM:**
- `sublime_git_tools::RepoError` → `CliError::Git`
- `sublime_pkg_tools::error::Error` → Várias categorias
- `std::io::Error` → `CliError::Io`
- `serde_json::Error` → `CliError::Execution`

### 4.2 Output & JsonResponse

#### Output Struct
**Arquivo**: `crates/cli/src/output/mod.rs`

```rust
pub struct Output {
    format: OutputFormat,
    writer: RefCell<Box<dyn Write + Send>>,
    no_color: bool,
}
```

#### JsonResponse Struct
**Arquivo**: `crates/cli/src/output/json.rs`

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JsonResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

**Formato JSON:**
```json
// Success
{
  "success": true,
  "data": { ... }
}

// Error
{
  "success": false,
  "error": "Error message"
}
```

### 4.3 Args Structs Principais

#### InitArgs
```rust
pub struct InitArgs {
    pub changeset_path: PathBuf,
    pub environments: Option<Vec<String>>,
    pub default_env: Option<Vec<String>>,
    pub strategy: Option<String>,
    pub registry: String,
    pub config_format: Option<String>,
    pub force: bool,
    pub non_interactive: bool,
}
```

#### ChangesetCreateArgs
```rust
pub struct ChangesetCreateArgs {
    pub bump: Option<String>,
    pub env: Option<Vec<String>>,
    pub branch: Option<String>,
    pub message: Option<String>,
    pub packages: Option<Vec<String>>,
    pub non_interactive: bool,
}
```

#### BumpArgs
```rust
pub struct BumpArgs {
    pub dry_run: bool,
    pub execute: bool,
    pub snapshot: bool,
    pub snapshot_format: Option<String>,
    pub prerelease: Option<String>,
    pub packages: Option<Vec<String>>,
    pub git_tag: bool,
    pub git_push: bool,
    pub git_commit: bool,
    pub no_changelog: bool,
    pub no_archive: bool,
    pub force: bool,
    pub show_diff: bool,
}
```

---

## 5. ARQUITETURA napi-rs: FUNDAMENTOS

### 5.1 O que é napi-rs?

napi-rs é um framework para criar addons nativos Node.js em Rust usando Node-API (N-API).

**Características principais:**
- **ABI-stable**: Binários funcionam em múltiplas versões Node.js
- **Type-safe**: Gera TypeScript definitions automaticamente
- **Zero-config**: Build simples com `napi build`
- **Cross-platform**: Suporta macOS, Linux, Windows, WASM
- **Async-first**: Rust async fn ↔ JavaScript Promises

### 5.2 Anatomia de um Projeto napi-rs

#### Cargo.toml Essencial
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "3", features = ["async", "tokio_rt", "serde-json"] }
napi-derive = "3"

[build-dependencies]
napi-build = "3"
```

#### build.rs
```rust
fn main() {
    napi_build::setup();
}
```

### 5.3 Conversão de Tipos Automática

| Rust | JavaScript | napi-rs |
|------|------------|---------|
| `String` | `string` | Automático |
| `i32`, `f64` | `number` | Automático |
| `bool` | `boolean` | Automático |
| `Vec<T>` | `Array<T>` | Automático |
| `Option<T>` | `T \| null \| undefined` | Automático |
| Custom struct | `interface` | `#[napi(object)]` |
| `PathBuf` | `string` | **MANUAL** |
| `Result<T,E>` | `Promise<T>` | Automático |

### 5.4 Async Functions

#### Rust
```rust
#[napi]
pub async fn my_async_fn(arg: String) -> napi::Result<String> {
    let result = tokio::fs::read_to_string(arg).await?;
    Ok(result)
}
```

#### TypeScript gerado
```typescript
export function myAsyncFn(arg: string): Promise<string>
```

---

## 6. ESTRATÉGIA DE IMPLEMENTAÇÃO

### 6.1 Decisões Arquiteturais Principais

#### ❶ Onde criar o crate?

**Decisão**: Criar novo crate `crates/node/`

**Rationale:**
- Separação clara: bindings separados da lógica CLI
- Facilita builds independentes
- Permite testes isolados
- Segue pattern do projeto (crates modulares)

#### ❷ Como integrar com package existente?

**Decisão**: Build do crate `crates/node/` gera binaries que vão para `packages/workspace-tools/src/`

**Rationale:**
- Mantém package NPM existente
- Adiciona novas funções ao lado das existentes
- Versioning unificado

#### ❸ JSON String vs Objetos JS?

**Decisão**: Retornar JSON como `String` em todas as funções

**Assinatura napi:**
```rust
#[napi]
pub async fn changeset_add(params: ChangesetAddParams) -> napi::Result<String>
```

**Rationale:**
- ✅ Simples: evita conversão complexa
- ✅ Universal: JSON funciona everywhere
- ✅ Performance: uma serialização apenas
- ✅ Testável: fácil comparar strings

#### ❹ Como capturar output?

**Padrão de implementação:**
```rust
pub async fn some_command(params: Params) -> napi::Result<String> {
    let args = convert_params_to_args(params)?;
    
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let output = Output::new(OutputFormat::Json, cursor, true);
    
    execute_something(&args, &output, root, config_path)
        .await
        .map_err(convert_cli_error)?;
    
    String::from_utf8(buffer)
        .map_err(|e| napi::Error::from_reason(format!("UTF-8 error: {e}")))
}
```

---

## 7. ARQUITETURA DETALHADA PROPOSTA

### 7.1 Estrutura de Diretórios Completa

```
workspace-node-tools/
├── crates/
│   ├── node/                           # ← NOVO CRATE
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── src/
│   │   │   ├── lib.rs                  # Entry point
│   │   │   ├── error.rs                # CliError → napi::Error
│   │   │   ├── utils/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── output.rs           # Buffer capture
│   │   │   │   └── conversion.rs       # Type conversions
│   │   │   ├── types/                  # napi param types
│   │   │   │   ├── mod.rs
│   │   │   │   ├── init.rs
│   │   │   │   ├── config.rs
│   │   │   │   ├── changeset.rs
│   │   │   │   ├── bump.rs
│   │   │   │   ├── upgrade.rs
│   │   │   │   ├── audit.rs
│   │   │   │   ├── changes.rs
│   │   │   │   └── clone.rs
│   │   │   └── commands/
│   │   │       ├── mod.rs
│   │   │       ├── init.rs
│   │   │       ├── config.rs
│   │   │       ├── changeset.rs
│   │   │       ├── bump.rs
│   │   │       ├── upgrade.rs
│   │   │       ├── audit.rs
│   │   │       ├── changes.rs
│   │   │       └── clone.rs
│   │   └── tests/
│   ├── cli/
│   ├── git/
│   ├── pkg/
│   └── standard/
└── packages/
    └── workspace-tools/
        ├── package.json
        ├── src/
        │   ├── binding.d.ts            # AUTO-GENERATED
        │   ├── binding.js              # AUTO-GENERATED
        │   └── index.ts                # Updated
        └── __test__/
            ├── init.test.ts
            ├── changeset.test.ts
            └── ...
```

### 7.2 Cargo.toml do Novo Crate

**`crates/node/Cargo.toml`:**
```toml
[package]
name = "sublime_node_tools"
version = "0.0.1"
edition = "2024"
authors = ["WebSublime"]
license = "MIT"
repository = "https://github.com/websublime/workspace-tools"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "3.0.0", features = ["async", "tokio_rt", "serde-json"] }
napi-derive = "3.0.0"

sublime_cli_tools = { workspace = true }
sublime_git_tools = { workspace = true }
sublime_pkg_tools = { workspace = true }
sublime_standard_tools = { workspace = true }

serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
tokio = { workspace = true, features = ["full"] }

[build-dependencies]
napi-build = "3.0.0"

[lints]
workspace = true
```

### 7.3 package.json (Updated)

```json
{
  "name": "@websublime/workspace-tools",
  "version": "2.0.0",
  "scripts": {
    "build-binding": "napi build -o=./src --manifest-path ../../crates/node/Cargo.toml --platform --package sublime_node_tools",
    "build-binding:release": "pnpm build-binding --release",
    "test": "ava"
  }
}
```

---

## 8. ESPECIFICAÇÃO DE CADA FUNÇÃO

### 8.1 Função Exemplo: `init`

#### Parâmetros JavaScript
```typescript
interface InitParams {
  root?: string;
  changesetPath: string;
  environments?: string[];
  defaultEnv?: string[];
  strategy?: "independent" | "unified";
  registry?: string;
  configFormat?: "json" | "yaml" | "toml";
  force?: boolean;
  nonInteractive?: boolean;
}
```

#### Implementação Rust

**`crates/node/src/types/init.rs`:**
```rust
use napi_derive::napi;

#[napi(object)]
#[derive(Debug)]
pub struct InitParams {
    pub root: Option<String>,
    pub changeset_path: String,
    pub environments: Option<Vec<String>>,
    pub default_env: Option<Vec<String>>,
    pub strategy: Option<String>,
    pub registry: Option<String>,
    pub config_format: Option<String>,
    pub force: Option<bool>,
    pub non_interactive: Option<bool>,
}
```

**`crates/node/src/commands/init.rs`:**
```rust
use napi_derive::napi;
use std::path::{Path, PathBuf};
use sublime_cli_tools::cli::commands::InitArgs;
use sublime_cli_tools::commands::init::execute_init;
use sublime_cli_tools::output::OutputFormat;

use crate::error::convert_cli_error;
use crate::types::init::InitParams;

#[napi]
pub async fn init(params: InitParams) -> napi::Result<String> {
    let args = InitArgs {
        changeset_path: PathBuf::from(params.changeset_path),
        environments: params.environments,
        default_env: params.default_env,
        strategy: params.strategy,
        registry: params.registry.unwrap_or_else(|| "https://registry.npmjs.org".to_string()),
        config_format: params.config_format,
        force: params.force.unwrap_or(false),
        non_interactive: params.non_interactive.unwrap_or(false),
    };
    
    let root = params.root.as_deref().unwrap_or(".");
    let root_path = Path::new(root);
    
    execute_init(&args, root_path, OutputFormat::Json)
        .await
        .map_err(convert_cli_error)?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Workspace initialized successfully"
    }).to_string())
}
```

### 8.2 Função Exemplo: `changesetAdd`

**`crates/node/src/commands/changeset.rs`:**
```rust
use std::io::Cursor;
use sublime_cli_tools::commands::changeset::add::execute_add;
use sublime_cli_tools::output::{Output, OutputFormat};

#[napi]
pub async fn changeset_add(params: ChangesetAddParams) -> napi::Result<String> {
    let args = ChangesetCreateArgs {
        bump: params.bump,
        env: params.env,
        branch: params.branch,
        message: params.message,
        packages: params.packages,
        non_interactive: params.non_interactive.unwrap_or(false),
    };
    
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let output = Output::new(OutputFormat::Json, cursor, true);
        
        execute_add(&args, &output, root, config_path)
            .await
            .map_err(convert_cli_error)?;
    }
    
    String::from_utf8(buffer)
        .map_err(|e| napi::Error::from_reason(format!("UTF-8 error: {e}")))
}
```

### 8.3 Resumo de Todas as Funções

| Função | Params | Pattern |
|--------|--------|---------|
| `init` | `InitParams` | Legacy |
| `configShow` | `ConfigShowParams` | Legacy |
| `configValidate` | `ConfigValidateParams` | Legacy |
| `changesetAdd` | `ChangesetAddParams` | Moderno |
| `changesetUpdate` | `ChangesetUpdateParams` | Moderno |
| `changesetList` | `ChangesetListParams` | Moderno |
| `changesetShow` | `ChangesetShowParams` | Moderno |
| `changesetRemove` | `ChangesetRemoveParams` | Moderno |
| `changesetHistory` | `ChangesetHistoryParams` | Moderno |
| `changesetCheck` | `ChangesetCheckParams` | Moderno |
| `bumpPreview` | `BumpParams` | Moderno |
| `bumpApply` | `BumpParams` | Moderno |
| `bumpSnapshot` | `BumpParams` | Moderno |
| `upgradeCheck` | `UpgradeCheckParams` | Moderno |
| `upgradeApply` | `UpgradeApplyParams` | Moderno |
| `backupList` | `BackupListParams` | Moderno |
| `backupRestore` | `BackupRestoreParams` | Moderno |
| `backupClean` | `BackupCleanParams` | Moderno |
| `audit` | `AuditParams` | Moderno |
| `changes` | `ChangesParams` | Moderno |
| `clone` | `CloneParams` | Legacy |

---

## 9. PADRÕES DE CÓDIGO E IMPLEMENTAÇÃO

### 9.1 Error Conversion

**`crates/node/src/error.rs`:**
```rust
use napi::Error as NapiError;
use sublime_cli_tools::error::CliError;

pub fn convert_cli_error(error: CliError) -> NapiError {
    let kind = error.kind();
    let message = format!("[{kind}] {error}");
    
    let status = match error {
        CliError::Configuration(_) => napi::Status::InvalidArg,
        CliError::Validation(_) => napi::Status::InvalidArg,
        CliError::Execution(_) => napi::Status::GenericFailure,
        CliError::Git(_) => napi::Status::GenericFailure,
        CliError::Package(_) => napi::Status::InvalidArg,
        CliError::Io(_) => napi::Status::GenericFailure,
        CliError::Network(_) => napi::Status::GenericFailure,
        CliError::User(_) => napi::Status::Cancelled,
    };
    
    NapiError::new(status, message)
}
```

### 9.2 lib.rs Entry Point

```rust
#![warn(missing_docs)]
#![deny(clippy::all)]

mod error;
mod types;
mod commands;
mod utils;

pub use commands::init::init;
pub use commands::config::{config_show, config_validate};
pub use commands::changeset::{
    changeset_add,
    changeset_update,
    changeset_list,
    changeset_show,
    changeset_remove,
    changeset_history,
    changeset_check,
};
pub use commands::bump::{
    bump_preview,
    bump_apply,
    bump_snapshot,
};
pub use commands::upgrade::{
    upgrade_check,
    upgrade_apply,
    backup_list,
    backup_restore,
    backup_clean,
};
pub use commands::audit::audit;
pub use commands::changes::changes;
pub use commands::clone::clone;
```

---

## 10. ERROR HANDLING COMPLETO

### 10.1 Mapeamento de Erros

| CliError | JavaScript | Status |
|----------|------------|--------|
| `Configuration` | `[ConfigurationError]` | `InvalidArg` |
| `Validation` | `[ValidationError]` | `InvalidArg` |
| `Git` | `[GitError]` | `GenericFailure` |
| `Package` | `[PackageError]` | `InvalidArg` |
| `Io` | `[IoError]` | `GenericFailure` |
| `Network` | `[NetworkError]` | `GenericFailure` |
| `User` | `[UserError]` | `Cancelled` |
| `Execution` | `[ExecutionError]` | `GenericFailure` |

### 10.2 JavaScript Error Handling

```javascript
try {
    const result = await changesetAdd({
        bump: 'minor',
        nonInteractive: true
    });
    const data = JSON.parse(result);
    console.log('Success:', data);
    
} catch (error) {
    if (error.message.includes('[GitError]')) {
        console.error('Git operation failed');
    } else if (error.message.includes('[ValidationError]')) {
        console.error('Invalid parameters');
    }
    process.exit(1);
}
```

---

## 11. TESTING STRATEGY

### 11.1 Rust Unit Tests

**Localização**: `crates/node/src/**/*.rs`  
**Coverage target**: 100%

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_git_error() {
        let cli_err = CliError::git("Repository not found");
        let napi_err = convert_cli_error(cli_err);
        assert!(format!("{napi_err}").contains("GitError"));
    }
}
```

### 11.2 Node.js Integration Tests

**Localização**: `packages/workspace-tools/__test__/`  
**Coverage target**: >90%

```typescript
import test from 'ava';
import { init, changesetAdd } from '../src/index.js';

test('init - creates workspace config', async (t) => {
    const result = await init({
        changesetPath: '.changesets',
        nonInteractive: true
    });
    
    const data = JSON.parse(result);
    t.is(data.success, true);
});
```

---

## 12. BUILD, DISTRIBUTION & CI/CD

### 12.1 Build Commands

```bash
# Build bindings (debug)
cd packages/workspace-tools
pnpm build-binding

# Build bindings (release)
pnpm build-binding:release

# Run tests
pnpm test
```

### 12.2 GitHub Actions Workflow

```yaml
name: Build Node Bindings

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        settings:
          - host: macos-13
            target: x86_64-apple-darwin
          - host: macos-13
            target: aarch64-apple-darwin
          - host: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - host: windows-latest
            target: x86_64-pc-windows-msvc
    
    runs-on: ${{ matrix.settings.host }}
    
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build
        run: |
          cd packages/workspace-tools
          pnpm install
          pnpm build-binding:release
      
      - name: Test
        run: pnpm test
```

---

## 13. EXEMPLOS DE USO COMPLETOS

### 13.1 Release Workflow

```javascript
import { changesetList, bumpPreview, bumpApply } from '@websublime/workspace-tools';

async function releaseWorkflow() {
    // 1. List changesets
    const listResult = await changesetList({});
    const listData = JSON.parse(listResult);
    
    if (listData.changesets.length === 0) {
        console.log('No changesets found');
        return;
    }
    
    // 2. Preview bumps
    const previewResult = await bumpPreview({ showDiff: true });
    const previewData = JSON.parse(previewResult);
    
    console.log('Packages to bump:');
    for (const pkg of previewData.packages) {
        console.log(`  ${pkg.name}: ${pkg.currentVersion} → ${pkg.newVersion}`);
    }
    
    // 3. Apply bumps
    const applyResult = await bumpApply({
        execute: true,
        gitCommit: true,
        gitTag: true
    });
    
    console.log('✅ Version bumps applied');
}

releaseWorkflow();
```

### 13.2 TypeScript Example

```typescript
import { init, changesetAdd, InitParams } from '@websublime/workspace-tools';

const params: InitParams = {
    changesetPath: '.changesets',
    nonInteractive: true
};

const result = await init(params);
const data = JSON.parse(result);
console.log('Initialized:', data);
```

---

## 14. PERFORMANCE & OPTIMIZATION

### 14.1 Expected Performance

| Operation | CLI Process | napi Binding | Speedup |
|-----------|-------------|--------------|---------|
| `init` | ~200ms | ~50ms | 4x |
| `changesetAdd` | ~150ms | ~30ms | 5x |
| `bumpPreview` | ~500ms | ~100ms | 5x |

**Key advantage**: Eliminação do process spawn overhead (~50-100ms)

---

## 15. ROADMAP DE IMPLEMENTAÇÃO

### Phase 1: Foundation (Semana 1)
- ✅ Criar estrutura `crates/node/`
- ✅ Configurar Cargo.toml
- ✅ Implementar error.rs
- ✅ Implementar utils/
- ✅ Setup CI/CD básico

### Phase 2: POC (Semana 2)
- ✅ Implementar init, configShow, configValidate
- ✅ Testes Node.js
- ✅ Verificar TypeScript defs

### Phase 3: Changeset (Semana 3)
- ✅ Implementar 7 funções changeset
- ✅ Testes completos
- ✅ Exemplos

### Phase 4: Bump (Semana 4)
- ✅ Implementar 3 funções bump
- ✅ Testes
- ✅ Workflow examples

### Phase 5: Upgrade (Semana 4-5)
- ✅ Implementar 5 funções upgrade
- ✅ Testes
- ✅ Documentation

### Phase 6: Remaining (Semana 5)
- ✅ Implementar audit, changes, clone
- ✅ Testes
- ✅ Examples

### Phase 7: Polish (Semana 6)
- ✅ Code review
- ✅ 100% test coverage
- ✅ Documentation completa
- ✅ Performance benchmarks

### Phase 8: Release (Semana 7)
- ✅ GitHub release
- ✅ npm publish
- ✅ Monitor feedback

**TOTAL**: 6-7 semanas

---

## 16. RISCOS, MITIGAÇÕES E DECISÕES

### 16.1 Riscos Técnicos

#### Risco 1: Output Capture Complexity
**Mitigação**: Usar `Vec<u8>` + `Cursor`, pattern estabelecido

#### Risco 2: Breaking Changes no CLI
**Mitigação**: Mesmo repo = mudanças atômicas, testes detectam

#### Risco 3: Platform-Specific Issues
**Mitigação**: CI testa todas plataformas, napi-rs maduro

### 16.2 Decisões Arquiteturais

#### JSON String vs Objetos JS
**Escolha**: JSON String  
**Rationale**: Simples, universal, performante

#### Novo Crate vs Módulo
**Escolha**: Novo crate `crates/node/`  
**Rationale**: Separação clara, builds independentes

#### Integração com Package Existente
**Escolha**: Expandir `@websublime/workspace-tools`  
**Rationale**: Unified interface, melhor UX

---

## 17. APÊNDICES

### Apêndice A: Comandos Úteis

```bash
# Build
pnpm build-binding:release

# Test
pnpm test

# Clippy
cargo clippy --all-targets

# Coverage
cargo tarpaulin --out Html

# Publish
npm publish --access public
```

### Apêndice B: Checklist de Release

- [ ] Clippy 100%
- [ ] Tests 100% (Rust)
- [ ] Tests >90% (Node.js)
- [ ] Documentation completa
- [ ] CHANGELOG atualizado
- [ ] Build todas plataformas
- [ ] npm publish

---

## CONCLUSÃO

Este relatório documenta uma estratégia **completa e implementável** para criar Node.js bindings usando napi-rs para 21 funções execute do CLI.

### Benefícios Finais

✅ **Reuso 100%**: Zero duplicação  
✅ **Type-safe**: TypeScript defs automáticas  
✅ **Cross-platform**: macOS, Linux, Windows  
✅ **Performance**: 4-5x speedup vs CLI  
✅ **Maintainable**: Padrões consistentes  

### Estimativas

- **Tempo**: 6-7 semanas
- **LOC**: ~4000-5000
- **Complexidade**: Média
- **Risco**: Baixo

---

**Fim do Relatório**
