# RELATÓRIO COMPLETO: Node.js Bindings com napi-rs para Sublime Workspace CLI Tools

**Projeto**: workspace-node-tools  
**Objetivo**: Criar bindings Node.js para expor 23 funções `execute_*` do CLI  
**Tecnologia**: napi-rs (versão 3.x)  
**Data**: 2025-01-18  
**Última Atualização**: 2025-01-20  
**Status**: Research & Planning

---

## ÍNDICE

1. [Executive Summary](#1-executive-summary)
2. [Estado Atual do Projeto](#2-estado-atual-do-projeto)
3. [Inventário Completo de Funções Execute](#3-inventário-completo-de-funções-execute)
4. [Análise Detalhada do Código Existente](#4-análise-detalhada-do-código-existente)
5. [Arquitetura napi-rs: Fundamentos](#5-arquitetura-napi-rs-fundamentos)
6. [Estratégia de Implementação](#6-estratégia-de-implementação)
7. [API vs CLI: Restrições e Simplificações](#7-api-vs-cli-restrições-e-simplificações)
8. [Sistema de Erros e Códigos](#8-sistema-de-erros-e-códigos)
9. [Validação de Parâmetros](#9-validação-de-parâmetros)
10. [Arquitetura Detalhada Proposta](#10-arquitetura-detalhada-proposta)
11. [Especificação de Cada Função](#11-especificação-de-cada-função)
12. [Padrões de Código e Implementação](#12-padrões-de-código-e-implementação)
13. [Testing Strategy](#13-testing-strategy)
14. [Build, Distribution & CI/CD](#14-build-distribution--cicd)
15. [Exemplos de Uso Completos](#15-exemplos-de-uso-completos)
16. [Performance & Optimization](#16-performance--optimization)
17. [Roadmap de Implementação](#17-roadmap-de-implementação)
18. [Riscos, Mitigações e Decisões](#18-riscos-mitigações-e-decisões)
19. [Apêndices](#19-apêndices)

---

## 1. EXECUTIVE SUMMARY

### 1.1 Contexto

O projeto workspace-node-tools é um conjunto de ferramentas Rust para gerenciamento de monorepos Node.js. Atualmente possui:
- **CLI completo** (`sublime_cli_tools`) com 23 funções `execute_*`
- **3 crates base**: `sublime_git_tools`, `sublime_pkg_tools`, `sublime_standard_tools`
- **1 package Node.js existente**: `@websublime/workspace-tools` (v1.0.2) com bindings básicos para git e standard tools

### 1.2 Objetivo

Expandir os bindings Node.js para incluir **23 funções execute do CLI**, permitindo que aplicações Node.js/TypeScript usem toda a funcionalidade de gerenciamento de workspace programaticamente, sem depender do CLI de terminal.

### 1.3 Escopo

**Funções a implementar (23):**
- **Init** (1): `init`
- **Config** (2): `configShow`, `configValidate`
- **Changeset** (7): `changesetAdd`, `changesetUpdate`, `changesetList`, `changesetShow`, `changesetRemove`, `changesetHistory`, `changesetCheck`
- **Bump** (3): `bumpPreview`, `bumpApply`, `bumpSnapshot`
- **Upgrade** (5): `upgradeCheck`, `upgradeApply`, `backupList`, `backupRestore`, `backupClean`
- **Audit** (1): `audit`
- **Changes** (1): `changes`
- **Clone** (1): `clone`
- **Status** (1): `status`
- **Execute** (1): `execute`

### 1.4 Benefícios

✅ **Reuso 100% da lógica**: Zero duplicação, chamamos execute functions diretamente  
✅ **Type-safe**: TypeScript definitions geradas automaticamente via `#[napi(object)]`  
✅ **Cross-platform**: macOS, Linux, Windows (ARM64 + x64)  
✅ **Performance**: Nativo, sem overhead de spawnar processos CLI  
✅ **Developer-friendly**: Objectos JS nativos, sem `JSON.parse()`  
✅ **Error Codes**: Códigos estruturados estilo Node.js (EVALIDATION, EGIT, etc.)  
✅ **Maintainable**: Mudanças no CLI refletem automaticamente nos bindings  

### 1.5 Decisões Arquiteturais Principais

#### ❶ Objectos JS Nativos (não JSON String)

Cada função napi retorna `Promise<ApiResponse<T>>` com objectos JavaScript nativos, gerados automaticamente pelo `#[napi(object)]`.

**Rationale:**
- ✅ Type-safe: TypeScript types gerados automaticamente
- ✅ Sem `JSON.parse()`: Melhor DX e performance
- ✅ Documentação automática: JSDoc comments propagados
- ✅ Consistência: O binding actual já usa `#[napi]` para classes

#### ❷ Códigos de Erro Estruturados

Erros seguem padrão Node.js com códigos como `EVALIDATION`, `EGIT`, `ECONFIG`.

#### ❸ Validação na Camada NAPI

Parâmetros são validados antes de chamar funções CLI, proporcionando feedback imediato.

#### ❹ Timeout Configurável para Execute

Suporta timeout via `repo.config` (default) com override por parâmetro.

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

| # | Categoria | Função Rust | Função JS | Arquivo | Args Struct |
|---|-----------|-------------|-----------|---------|-------------|
| 1 | Init | `execute_init` | `init` | `commands/init.rs` | `InitArgs` |
| 2 | Config | `execute_show` | `configShow` | `commands/config.rs` | `ConfigShowArgs` |
| 3 | Config | `execute_validate` | `configValidate` | `commands/config.rs` | `ConfigValidateArgs` |
| 4 | Changeset | `execute_add` | `changesetAdd` | `commands/changeset/add.rs` | `ChangesetCreateArgs` |
| 5 | Changeset | `execute_update` | `changesetUpdate` | `commands/changeset/update.rs` | `ChangesetUpdateArgs` |
| 6 | Changeset | `execute_list` | `changesetList` | `commands/changeset/list.rs` | `ChangesetListArgs` |
| 7 | Changeset | `execute_show` | `changesetShow` | `commands/changeset/show.rs` | `ChangesetShowArgs` |
| 8 | Changeset | `execute_remove` | `changesetRemove` | `commands/changeset/remove.rs` | `ChangesetDeleteArgs` |
| 9 | Changeset | `execute_history` | `changesetHistory` | `commands/changeset/history.rs` | `ChangesetHistoryArgs` |
| 10 | Changeset | `execute_check` | `changesetCheck` | `commands/changeset/check.rs` | `ChangesetCheckArgs` |
| 11 | Bump | `execute_bump_preview` | `bumpPreview` | `commands/bump/preview.rs` | `BumpArgs` |
| 12 | Bump | `execute_bump_apply` | `bumpApply` | `commands/bump/execute.rs` | `BumpArgs` |
| 13 | Bump | `execute_bump_snapshot` | `bumpSnapshot` | `commands/bump/snapshot.rs` | `BumpArgs` |
| 14 | Upgrade | `execute_upgrade_check` | `upgradeCheck` | `commands/upgrade/check.rs` | `UpgradeCheckArgs` |
| 15 | Upgrade | `execute_upgrade_apply` | `upgradeApply` | `commands/upgrade/apply.rs` | `UpgradeApplyArgs` |
| 16 | Upgrade | `execute_backup_list` | `backupList` | `commands/upgrade/rollback.rs` | `UpgradeBackupListArgs` |
| 17 | Upgrade | `execute_backup_restore` | `backupRestore` | `commands/upgrade/rollback.rs` | `UpgradeBackupRestoreArgs` |
| 18 | Upgrade | `execute_backup_clean` | `backupClean` | `commands/upgrade/rollback.rs` | `UpgradeBackupCleanArgs` |
| 19 | Audit | `execute_audit` | `audit` | `commands/audit/comprehensive.rs` | `AuditArgs` |
| 20 | Changes | `execute_changes` | `changes` | `commands/changes.rs` | `ChangesArgs` |
| 21 | Clone | `execute_clone` | `clone` | `commands/clone.rs` | `CloneArgs` |
| 22 | Status | `execute_status` | `status` | `commands/status.rs` | `StatusArgs` |
| 23 | Execute | `execute_execute` | `execute` | `commands/execute.rs` | `ExecuteArgs` |

**Total**: 23 funções

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

#### Padrão B: Moderno (Changeset, Bump, Upgrade, Audit, Changes, Status, Execute)
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
    pub force: bool,  // Overwrites existing changeset for the branch
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
    pub always_archive: bool,  // Forces archiving even for prerelease versions
    pub force: bool,
    pub show_diff: bool,
}
```

**Prerelease vs Snapshot Modes:**

| Mode | Flag | Purpose | SemVer | Persisted |
|------|------|---------|--------|-----------|
| Prerelease | `--prerelease <TAG>` | Official pre-releases (alpha, beta, rc) | ✅ Yes | ✅ Yes |
| Snapshot | `--snapshot` | Temporary versions for testing/CI | ❌ No | ❌ No |

**Prerelease Examples:**
- `--prerelease alpha` → `1.2.3` → `1.3.0-alpha.0`
- `--prerelease beta` → `1.3.0-alpha.0` → `1.3.0-beta.0`
- `--prerelease rc` → `1.3.0-beta.1` → `1.3.0-rc.0`

**Snapshot Format Variables:**
- `{version}` - Base version (e.g., `1.2.3`)
- `{branch}` - Git branch name (sanitized)
- `{commit}` - Full commit hash
- `{short_commit}` - Short commit hash (7 chars)
- `{timestamp}` - Unix timestamp

#### StatusArgs
```rust
pub struct StatusArgs {
    // Sem argumentos adicionais - usa opções globais (--format, --root, etc.)
}
```

#### ExecuteArgs
```rust
pub struct ExecuteArgs {
    pub cmd: String,
    pub filter_package: Option<Vec<String>>,
    pub affected: bool,
    pub since: Option<String>,
    pub until: Option<String>,
    pub branch: Option<String>,
    pub parallel: bool,
    pub args: Vec<String>,
}
```

### 4.4 Gestão de Configuração

**Arquivo**: `crates/cli/src/commands/mod.rs`

O CLI utiliza `find_and_load_config` para carregar automaticamente a configuração do workspace:

```rust
pub async fn find_and_load_config(
    root: &Path,
    config_path: Option<&Path>,
) -> Result<Option<PackageToolsConfig>> {
    // ...
    let config_files = vec![
        root.join("repo.config.toml"),
        root.join("repo.config.json"),
        root.join("repo.config.yaml"),
        root.join("repo.config.yml"),
    ];
    // ...
}
```

**Ficheiros de configuração suportados:**
1. `repo.config.toml` (preferido)
2. `repo.config.json`
3. `repo.config.yaml`
4. `repo.config.yml`

**Comportamento:**
- Se `config_path` é fornecido, usa esse ficheiro específico
- Caso contrário, procura pelos ficheiros na ordem acima
- Se nenhum for encontrado, retorna `None` e usa defaults

### 4.5 Timeout Existente no Config

O `repo.config` já suporta timeout para operações de registry:

```rust
// crates/pkg/src/config/upgrade.rs
pub struct RegistryConfig {
    /// HTTP request timeout in seconds.
    /// Default: 30
    pub timeout_secs: u64,
    
    /// Number of retry attempts for failed requests.
    /// Default: 3
    pub retry_attempts: u64,
    
    /// Delay between retries in milliseconds.
    /// Default: 1000
    pub retry_delay_ms: u64,
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
napi = { version = "3", features = ["async", "tokio_rt"] }
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
| `Result<T,E>` | `Promise<T>` / throws | Automático |

### 5.4 `#[napi(object)]` - A Chave para Objectos Nativos

O decorator `#[napi(object)]` permite retornar structs Rust como objectos JavaScript nativos:

```rust
#[napi(object)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub path: String,
}

#[napi]
pub async fn get_packages() -> napi::Result<Vec<PackageInfo>> {
    Ok(vec![
        PackageInfo {
            name: "@scope/core".to_string(),
            version: "1.0.0".to_string(),
            path: "packages/core".to_string(),
        }
    ])
}
```

**TypeScript gerado automaticamente:**
```typescript
export interface PackageInfo {
  name: string
  version: string
  path: string
}

export function getPackages(): Promise<PackageInfo[]>
```

### 5.5 Async Functions

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

#### ❸ Objectos JS Nativos vs JSON String

**Decisão**: Retornar objectos JavaScript nativos via `#[napi(object)]`

**Rationale:**
- ✅ Type-safe: TypeScript types gerados automaticamente
- ✅ Sem `JSON.parse()`: Melhor DX
- ✅ Performance: Conversão directa Rust → JS
- ✅ Documentação: JSDoc propagado automaticamente
- ✅ Consistência: Mesmo padrão do binding existente

#### ❹ Como capturar output do CLI?

**Padrão de implementação:**
```rust
pub async fn some_command(params: Params) -> napi::Result<ApiResponse<SomeData>> {
    // 1. Validar parâmetros
    validate_params(&params)?;
    
    // 2. Converter para args do CLI
    let args = convert_params_to_args(&params);
    
    // 3. Executar com buffer de captura
    let mut buffer = Vec::new();
    {
        let output = Output::new(OutputFormat::Json, Cursor::new(&mut buffer), true);
        execute_something(&args, &output, root, config_path)
            .await
            .map_err(convert_cli_error)?;
    }
    
    // 4. Parse do JSON e conversão para struct NAPI
    let json_str = String::from_utf8(buffer)?;
    let cli_response: CliJsonResponse<SomeCliData> = serde_json::from_str(&json_str)?;
    
    // 5. Converter para resposta NAPI
    Ok(convert_to_api_response(cli_response))
}
```

---

## 7. API VS CLI: RESTRIÇÕES E SIMPLIFICAÇÕES

### 7.1 Parâmetros Fixos na API NAPI

A camada NAPI deve forçar certos parâmetros para garantir comportamento consistente em ambientes programáticos:

| Parâmetro | Valor Fixo | Justificação |
|-----------|------------|--------------|
| `non_interactive` | `true` | APIs não devem ter prompts interativos |
| `OutputFormat` | `Json` | Saída programática deve ser sempre estruturada |
| `no_color` | `true` | Não há terminal, cores são irrelevantes |

### 7.2 Gestão de Configuração na API

A API NAPI herda o comportamento de configuração do CLI:

1. **Detecção Automática**: Procura por `repo.config.{toml,json,yaml,yml}` no root
2. **Override Explícito**: Parâmetro `configPath` pode forçar um ficheiro específico
3. **Fallback**: Se não encontrar, usa `PackageToolsConfig::default()`

**Exemplo de uso:**
```typescript
// Usa detecção automática
await status({ root: '/path/to/project' });

// Override explícito
await status({ 
  root: '/path/to/project',
  configPath: '/path/to/custom/repo.config.toml'
});
```

### 7.3 Simplificação de Parâmetros

Para cada função NAPI, os seguintes parâmetros são **removidos** da interface:
- Não expostos: `non_interactive`, `format`, `no_color`
- Forçados internamente com valores adequados para API

**Exemplo - InitArgs CLI vs InitParams NAPI:**

```typescript
// CLI (todos os parâmetros)
interface InitArgs {
  changeset_path: string;
  environments?: string[];
  default_env?: string[];
  strategy?: string;
  registry?: string;
  config_format?: string;
  force?: boolean;
  non_interactive?: boolean;  // ← Removido na API
}

// NAPI (simplificado)
interface InitParams {
  root?: string;
  changesetPath: string;
  environments?: string[];
  defaultEnv?: string[];
  strategy?: string;
  registry?: string;
  configFormat?: string;
  force?: boolean;
  // non_interactive é sempre true internamente
}
```

### 7.4 Vantagens desta Abordagem

| Aspecto | CLI | API NAPI |
|---------|-----|----------|
| Interactividade | Suportada | Desabilitada |
| Formato output | Human/JSON | Objectos JS nativos |
| Cores terminal | Opcional | Desabilitado |
| Config path | Flag opcional | Parâmetro opcional |
| Complexidade | Alta | Simplificada |
| Type-safety | N/A | Total (TypeScript) |

---

## 8. SISTEMA DE ERROS E CÓDIGOS

### 8.1 Códigos de Erro Estilo Node.js

Adoptamos um sistema de códigos de erro inspirado no Node.js para consistência:

| Código | Significado | Mapeamento CliError |
|--------|-------------|---------------------|
| `ECONFIG` | Erro de configuração | `Configuration` |
| `EVALIDATION` | Erro de validação de parâmetros | `Validation` |
| `EEXEC` | Erro de execução | `Execution` |
| `EGIT` | Erro Git | `Git` |
| `EPKG` | Erro de package | `Package` |
| `ENOENT` | Ficheiro/path não encontrado | `Io` |
| `EIO` | Erro de I/O genérico | `Io` |
| `ENETWORK` | Erro de rede | `Network` |
| `EUSER` | Cancelado pelo utilizador | `User` |
| `ETIMEOUT` | Timeout excedido | Novo |

### 8.2 Estrutura de Erro NAPI

```rust
/// Structured error information for API responses.
#[napi(object)]
pub struct ErrorInfo {
    /// Error code following Node.js conventions (e.g., "EVALIDATION", "EGIT").
    pub code: String,
    
    /// Human-readable error message.
    pub message: String,
    
    /// Additional context about the error (optional).
    pub context: Option<String>,
    
    /// Original error kind from CLI (for debugging).
    pub kind: String,
}
```

### 8.3 Conversão de CliError para ErrorInfo

```rust
/// Converts a CliError to ErrorInfo with appropriate error code.
pub fn cli_error_to_info(error: &CliError) -> ErrorInfo {
    let (code, kind) = match error {
        CliError::Configuration(_) => ("ECONFIG", "Configuration"),
        CliError::Validation(_) => ("EVALIDATION", "Validation"),
        CliError::Execution(_) => ("EEXEC", "Execution"),
        CliError::Git(_) => ("EGIT", "Git"),
        CliError::Package(_) => ("EPKG", "Package"),
        CliError::Io(msg) => {
            if msg.contains("not found") || msg.contains("No such file") {
                ("ENOENT", "Io")
            } else {
                ("EIO", "Io")
            }
        }
        CliError::Network(_) => ("ENETWORK", "Network"),
        CliError::User(_) => ("EUSER", "User"),
    };
    
    ErrorInfo {
        code: code.to_string(),
        message: error.user_message(),
        context: None,
        kind: kind.to_string(),
    }
}
```

### 8.4 API Response Wrapper

```rust
/// Standard API response wrapper.
/// All NAPI functions return this structure for consistency.
#[napi(object)]
pub struct ApiResponse<T> {
    /// Whether the operation succeeded.
    pub success: bool,
    
    /// The response data (present when success is true).
    pub data: Option<T>,
    
    /// Error information (present when success is false).
    pub error: Option<ErrorInfo>,
}

impl<T> ApiResponse<T> {
    /// Creates a successful response with data.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    /// Creates a failed response with error info.
    pub fn failure(error: ErrorInfo) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}
```

### 8.5 TypeScript Gerado

```typescript
export interface ErrorInfo {
  /** Error code following Node.js conventions (e.g., "EVALIDATION", "EGIT") */
  code: string
  /** Human-readable error message */
  message: string
  /** Additional context about the error */
  context?: string
  /** Original error kind from CLI (for debugging) */
  kind: string
}

export interface ApiResponse<T> {
  /** Whether the operation succeeded */
  success: boolean
  /** The response data (present when success is true) */
  data?: T
  /** Error information (present when success is false) */
  error?: ErrorInfo
}
```

### 8.6 Uso em JavaScript/TypeScript

```typescript
const result = await status({ root: '/path/to/project' });

if (result.success) {
  console.log('Packages:', result.data.packages);
} else {
  const { code, message, context } = result.error;
  
  switch (code) {
    case 'ENOENT':
      console.error(`File not found: ${message}`);
      break;
    case 'EGIT':
      console.error(`Git error: ${message}`);
      break;
    case 'EVALIDATION':
      console.error(`Invalid parameters: ${message}`);
      break;
    default:
      console.error(`[${code}] ${message}`);
  }
  
  process.exit(1);
}
```

---

## 9. VALIDAÇÃO DE PARÂMETROS

### 9.1 Princípios de Validação

A validação ocorre na camada NAPI **antes** de chamar as funções CLI:

1. **Fail Fast**: Erros detectados imediatamente
2. **Mensagens Claras**: Contexto específico sobre o que falhou
3. **Type-safe**: TypeScript previne muitos erros em compile-time
4. **Consistente**: Todas as funções seguem o mesmo padrão

### 9.2 Estrutura de Validação

```rust
/// Validation error with context.
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub value: Option<String>,
}

impl ValidationError {
    pub fn required(field: &str) -> Self {
        Self {
            field: field.to_string(),
            message: format!("{field} is required"),
            value: None,
        }
    }
    
    pub fn invalid(field: &str, message: &str, value: Option<String>) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            value,
        }
    }
}

/// Converts ValidationError to ErrorInfo.
impl From<ValidationError> for ErrorInfo {
    fn from(err: ValidationError) -> Self {
        ErrorInfo {
            code: "EVALIDATION".to_string(),
            message: err.message,
            context: Some(format!("field: {}", err.field)),
            kind: "Validation".to_string(),
        }
    }
}
```

### 9.3 Validadores Comuns

```rust
/// Validation utilities for NAPI parameters.
pub mod validators {
    use super::*;
    
    /// Validates that a path exists.
    pub fn path_exists(field: &str, path: &str) -> Result<(), ValidationError> {
        if !std::path::Path::new(path).exists() {
            return Err(ValidationError::invalid(
                field,
                &format!("path does not exist: {path}"),
                Some(path.to_string()),
            ));
        }
        Ok(())
    }
    
    /// Validates that a string is not empty.
    pub fn not_empty(field: &str, value: &Option<String>) -> Result<(), ValidationError> {
        match value {
            Some(v) if v.trim().is_empty() => {
                Err(ValidationError::invalid(field, "cannot be empty", None))
            }
            _ => Ok(()),
        }
    }
    
    /// Validates bump type.
    pub fn bump_type(field: &str, value: &Option<String>) -> Result<(), ValidationError> {
        if let Some(bump) = value {
            match bump.as_str() {
                "major" | "minor" | "patch" | "none" => Ok(()),
                _ => Err(ValidationError::invalid(
                    field,
                    "must be one of: major, minor, patch, none",
                    Some(bump.clone()),
                )),
            }
        } else {
            Ok(())
        }
    }
    
    /// Validates timeout value.
    pub fn timeout(field: &str, value: Option<u64>) -> Result<(), ValidationError> {
        if let Some(timeout) = value {
            if timeout == 0 {
                return Err(ValidationError::invalid(
                    field,
                    "timeout must be greater than 0",
                    Some(timeout.to_string()),
                ));
            }
            if timeout > 3600 {
                return Err(ValidationError::invalid(
                    field,
                    "timeout cannot exceed 3600 seconds (1 hour)",
                    Some(timeout.to_string()),
                ));
            }
        }
        Ok(())
    }
}
```

### 9.4 Exemplo de Validação de Parâmetros

```rust
/// Validates ExecuteParams before calling CLI.
fn validate_execute_params(params: &ExecuteParams) -> Result<(), ErrorInfo> {
    // Required: root
    validators::path_exists("root", &params.root)?;
    
    // Required: cmd
    if params.cmd.trim().is_empty() {
        return Err(ValidationError::required("cmd").into());
    }
    
    // Optional: timeout
    validators::timeout("timeoutSecs", params.timeout_secs)?;
    
    // Mutual exclusion: filterPackage vs affected
    if params.filter_package.is_some() && params.affected == Some(true) {
        return Err(ValidationError::invalid(
            "filterPackage",
            "cannot be used with 'affected'",
            None,
        ).into());
    }
    
    Ok(())
}
```

---

## 10. ARQUITETURA DETALHADA PROPOSTA

### 10.1 Estrutura de Diretórios Completa

```
workspace-node-tools/
├── crates/
│   ├── node/                           # ← NOVO CRATE
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── src/
│   │   │   ├── lib.rs                  # Entry point, exports
│   │   │   ├── error.rs                # ErrorInfo, códigos, conversão
│   │   │   ├── validation.rs           # Validadores de parâmetros
│   │   │   ├── response.rs             # ApiResponse<T>
│   │   │   ├── types/                  # Structs NAPI (params + responses)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── common.rs           # Tipos comuns
│   │   │   │   ├── init.rs
│   │   │   │   ├── config.rs
│   │   │   │   ├── changeset.rs
│   │   │   │   ├── bump.rs
│   │   │   │   ├── upgrade.rs
│   │   │   │   ├── audit.rs
│   │   │   │   ├── changes.rs
│   │   │   │   ├── clone.rs
│   │   │   │   ├── status.rs
│   │   │   │   └── execute.rs
│   │   │   └── commands/               # Implementação das funções
│   │   │       ├── mod.rs
│   │   │       ├── init.rs
│   │   │       ├── config.rs
│   │   │       ├── changeset.rs
│   │   │       ├── bump.rs
│   │   │       ├── upgrade.rs
│   │   │       ├── audit.rs
│   │   │       ├── changes.rs
│   │   │       ├── clone.rs
│   │   │       ├── status.rs
│   │   │       └── execute.rs
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
        │   └── index.ts
        └── __test__/
            ├── init.test.ts
            ├── changeset.test.ts
            ├── status.test.ts
            ├── execute.test.ts
            └── ...
```

### 10.2 Cargo.toml do Novo Crate

**`crates/node/Cargo.toml`:**
```toml
[package]
name = "sublime_node_tools"
version = "0.0.1"
edition = "2024"
authors = ["WebSublime"]
license = "MIT"
repository = "https://github.com/websublime/workspace-tools"
description = "Node.js bindings for Sublime Workspace CLI Tools"

[lib]
crate-type = ["cdylib"]

[dependencies]
# NAPI
napi = { version = "3.0.0", features = ["async", "tokio_rt"] }
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
tokio = { workspace = true, features = ["full"] }

# Tracing
tracing = { workspace = true }

[build-dependencies]
napi-build = "3.0.0"

[lints]
workspace = true
```

### 10.3 Configuração de Timeout para Execute

**Proposta de adição ao `repo.config`:**

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

**Estrutura Rust correspondente:**
```rust
// crates/pkg/src/config/execute.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecuteConfig {
    /// Default timeout for execute commands in seconds.
    /// 0 means no timeout.
    /// Default: 300
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    
    /// Timeout per individual package in seconds.
    /// Default: 60
    #[serde(default = "default_per_package_timeout")]
    pub per_package_timeout_secs: u64,
    
    /// Maximum number of parallel executions.
    /// Default: 8
    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
}

fn default_timeout() -> u64 { 300 }
fn default_per_package_timeout() -> u64 { 60 }
fn default_max_parallel() -> usize { 8 }
```

**Comportamento de timeout:**
1. Config fornece valores default
2. Parâmetro na função pode fazer override
3. 0 = sem timeout

---

## 11. ESPECIFICAÇÃO DE CADA FUNÇÃO

### 11.1 Função: `status`

#### Parâmetros (TypeScript)
```typescript
interface StatusParams {
  /** Workspace root directory path */
  root: string
  /** Optional custom config file path */
  configPath?: string
}
```

#### Resposta (TypeScript)
```typescript
interface StatusData {
  repository: {
    kind: 'simple' | 'monorepo' | 'unknown'
    monorepoType?: 'npm' | 'yarn' | 'pnpm' | 'bun' | 'deno' | string
  }
  packageManager: {
    name: 'npm' | 'yarn' | 'pnpm' | 'bun' | 'jsr' | 'unknown'
    lockFile: string
  }
  branch?: {
    name: string
  }
  changesets: Array<{ id: string }>
  packages: Array<{
    name: string
    version: string
    path: string
  }>
}
```

#### Implementação Rust

**`crates/node/src/types/status.rs`:**
```rust
use napi_derive::napi;

/// Parameters for the status command.
#[napi(object)]
pub struct StatusParams {
    /// Workspace root directory path.
    pub root: String,
    /// Optional custom config file path.
    pub config_path: Option<String>,
}

/// Repository information.
#[napi(object)]
pub struct RepositoryInfo {
    /// Repository kind: "simple", "monorepo", or "unknown".
    pub kind: String,
    /// Monorepo type if applicable.
    pub monorepo_type: Option<String>,
}

/// Package manager information.
#[napi(object)]
pub struct PackageManagerInfo {
    /// Package manager name.
    pub name: String,
    /// Lock file name.
    pub lock_file: String,
}

/// Branch information.
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
    /// Repository information.
    pub repository: RepositoryInfo,
    /// Package manager information.
    pub package_manager: PackageManagerInfo,
    /// Current branch (if available).
    pub branch: Option<BranchInfo>,
    /// Pending changesets.
    pub changesets: Vec<ChangesetInfo>,
    /// Workspace packages.
    pub packages: Vec<PackageInfo>,
}
```

**`crates/node/src/commands/status.rs`:**
```rust
use napi_derive::napi;
use std::io::Cursor;
use std::path::PathBuf;

use crate::error::{cli_error_to_info, ErrorInfo};
use crate::response::ApiResponse;
use crate::types::status::{StatusData, StatusParams};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::StatusArgs;
use sublime_cli_tools::commands::status::execute_status;
use sublime_cli_tools::output::{Output, OutputFormat};

/// Validates status parameters.
fn validate_params(params: &StatusParams) -> Result<(), ErrorInfo> {
    validators::path_exists("root", &params.root)?;
    Ok(())
}

/// Get workspace status information.
///
/// Returns comprehensive information about the workspace including
/// repository type, package manager, Git branch, pending changesets,
/// and all workspace packages.
#[napi]
pub async fn status(params: StatusParams) -> napi::Result<ApiResponse<StatusData>> {
    // Validate parameters
    if let Err(error) = validate_params(&params) {
        return Ok(ApiResponse::failure(error));
    }
    
    let root = PathBuf::from(&params.root);
    let config_path = params.config_path.as_ref().map(PathBuf::from);
    
    // Execute CLI command with JSON output capture
    let mut buffer = Vec::new();
    {
        let output = Output::new(OutputFormat::Json, Cursor::new(&mut buffer), true);
        let args = StatusArgs {};
        
        if let Err(e) = execute_status(&args, &output, &root, config_path.as_deref()).await {
            return Ok(ApiResponse::failure(cli_error_to_info(&e)));
        }
    }
    
    // Parse CLI JSON response
    let json_str = String::from_utf8(buffer)
        .map_err(|e| napi::Error::from_reason(format!("UTF-8 error: {e}")))?;
    
    // Convert to NAPI response
    match parse_status_response(&json_str) {
        Ok(data) => Ok(ApiResponse::success(data)),
        Err(e) => Ok(ApiResponse::failure(e)),
    }
}

fn parse_status_response(json: &str) -> Result<StatusData, ErrorInfo> {
    // Parse and convert CLI response to NAPI types
    // Implementation details...
    todo!()
}
```

### 11.2 Função: `execute`

#### Parâmetros (TypeScript)
```typescript
interface ExecuteParams {
  /** Workspace root directory path */
  root: string
  /** Command to execute (npm:<script> or system command) */
  cmd: string
  /** Filter packages to run command on */
  filterPackage?: string[]
  /** Execute only on affected packages */
  affected?: boolean
  /** Since commit/branch/tag for affected detection */
  since?: string
  /** Until commit/branch/tag for affected detection */
  until?: string
  /** Compare against branch for affected detection */
  branch?: string
  /** Run commands in parallel */
  parallel?: boolean
  /** Additional arguments to pass to command */
  args?: string[]
  /** Timeout in seconds (overrides config) */
  timeoutSecs?: number
  /** Per-package timeout in seconds (overrides config) */
  perPackageTimeoutSecs?: number
}
```

#### Resposta (TypeScript)
```typescript
interface ExecuteData {
  command: string
  results: Array<{
    package: string
    success: boolean
    exitCode: number
    durationMs: number
    error?: string
  }>
  summary: {
    total: number
    succeeded: number
    failed: number
    totalDurationMs: number
  }
}
```

#### Implementação Rust

**`crates/node/src/types/execute.rs`:**
```rust
use napi_derive::napi;

/// Parameters for the execute command.
#[napi(object)]
pub struct ExecuteParams {
    /// Workspace root directory path.
    pub root: String,
    /// Command to execute (npm:<script> or system command).
    pub cmd: String,
    /// Filter packages to run command on.
    pub filter_package: Option<Vec<String>>,
    /// Execute only on affected packages.
    pub affected: Option<bool>,
    /// Since commit/branch/tag for affected detection.
    pub since: Option<String>,
    /// Until commit/branch/tag for affected detection.
    pub until: Option<String>,
    /// Compare against branch for affected detection.
    pub branch: Option<String>,
    /// Run commands in parallel.
    pub parallel: Option<bool>,
    /// Additional arguments to pass to command.
    pub args: Option<Vec<String>>,
    /// Timeout in seconds (overrides config). 0 = no timeout.
    pub timeout_secs: Option<u64>,
    /// Per-package timeout in seconds (overrides config).
    pub per_package_timeout_secs: Option<u64>,
}

/// Result for a single package execution.
#[napi(object)]
pub struct PackageExecutionResult {
    /// Package name.
    pub package: String,
    /// Whether execution succeeded.
    pub success: bool,
    /// Exit code from the command.
    pub exit_code: i32,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Error message if execution failed.
    pub error: Option<String>,
}

/// Execution summary.
#[napi(object)]
pub struct ExecuteSummary {
    /// Total number of packages.
    pub total: u32,
    /// Number of successful executions.
    pub succeeded: u32,
    /// Number of failed executions.
    pub failed: u32,
    /// Total execution duration in milliseconds.
    pub total_duration_ms: u64,
}

/// Execute command response data.
#[napi(object)]
pub struct ExecuteData {
    /// Command that was executed.
    pub command: String,
    /// Results for each package.
    pub results: Vec<PackageExecutionResult>,
    /// Execution summary.
    pub summary: ExecuteSummary,
}
```

**`crates/node/src/commands/execute.rs`:**
```rust
use napi_derive::napi;
use std::io::Cursor;
use std::path::PathBuf;

use crate::error::{cli_error_to_info, ErrorInfo};
use crate::response::ApiResponse;
use crate::types::execute::{ExecuteData, ExecuteParams};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::ExecuteArgs;
use sublime_cli_tools::commands::execute::execute_execute;
use sublime_cli_tools::output::{Output, OutputFormat};

/// Validates execute parameters.
fn validate_params(params: &ExecuteParams) -> Result<(), ErrorInfo> {
    // Required: root
    validators::path_exists("root", &params.root)?;
    
    // Required: cmd
    if params.cmd.trim().is_empty() {
        return Err(crate::validation::ValidationError::required("cmd").into());
    }
    
    // Validate timeouts
    validators::timeout("timeoutSecs", params.timeout_secs)?;
    validators::timeout("perPackageTimeoutSecs", params.per_package_timeout_secs)?;
    
    // Mutual exclusion: filterPackage vs affected
    if params.filter_package.is_some() && params.affected == Some(true) {
        return Err(crate::validation::ValidationError::invalid(
            "filterPackage",
            "cannot be used together with 'affected'",
            None,
        ).into());
    }
    
    Ok(())
}

/// Execute commands across workspace packages.
///
/// Runs npm scripts or system commands across all or filtered
/// workspace packages with optional parallel execution.
#[napi]
pub async fn execute(params: ExecuteParams) -> napi::Result<ApiResponse<ExecuteData>> {
    // Validate parameters
    if let Err(error) = validate_params(&params) {
        return Ok(ApiResponse::failure(error));
    }
    
    let root = PathBuf::from(&params.root);
    
    // Build CLI args
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
    
    // TODO: Apply timeout from params or config
    
    // Execute CLI command with JSON output capture
    let mut buffer = Vec::new();
    {
        let output = Output::new(OutputFormat::Json, Cursor::new(&mut buffer), true);
        
        if let Err(e) = execute_execute(&args, &output, &root).await {
            return Ok(ApiResponse::failure(cli_error_to_info(&e)));
        }
    }
    
    // Parse CLI JSON response
    let json_str = String::from_utf8(buffer)
        .map_err(|e| napi::Error::from_reason(format!("UTF-8 error: {e}")))?;
    
    // Convert to NAPI response
    match parse_execute_response(&json_str) {
        Ok(data) => Ok(ApiResponse::success(data)),
        Err(e) => Ok(ApiResponse::failure(e)),
    }
}

fn parse_execute_response(json: &str) -> Result<ExecuteData, ErrorInfo> {
    // Parse and convert CLI response to NAPI types
    // Implementation details...
    todo!()
}
```

### 11.3 Funções: `bumpPreview`, `bumpApply`, `bumpSnapshot`

#### Parâmetros (TypeScript)

```typescript
/** Parameters for bumpPreview - preview version changes without applying */
interface BumpPreviewParams {
  /** Workspace root directory path */
  root?: string;
  /** Optional custom config file path */
  configPath?: string;
  /** Filter to specific packages */
  packages?: string[];
  /** Show detailed version diffs */
  showDiff?: boolean;
}

/** Parameters for bumpApply - apply version changes */
interface BumpApplyParams {
  /** Workspace root directory path */
  root?: string;
  /** Optional custom config file path */
  configPath?: string;
  /** Filter to specific packages */
  packages?: string[];
  
  // Git operations
  /** Create a Git commit with version changes */
  gitCommit?: boolean;
  /** Create Git tags for releases (format: package@version) */
  gitTag?: boolean;
  /** Push Git tags to remote (requires gitTag) */
  gitPush?: boolean;
  
  // Prerelease support
  /** 
   * Pre-release tag for official pre-release versions.
   * Creates semver-compliant pre-release versions.
   * Options: "alpha", "beta", "rc" (or custom tag)
   * Example: "beta" → 1.2.3 → 1.3.0-beta.0
   */
  prerelease?: string;
  
  // Changelog/Archive control
  /** Skip changelog generation/updates */
  noChangelog?: boolean;
  /** Keep changesets active after bump (don't archive) */
  noArchive?: boolean;
  /** Force archiving even for prerelease versions */
  alwaysArchive?: boolean;
  
  // Behavior
  /** Skip confirmation prompts */
  force?: boolean;
}

/** Parameters for bumpSnapshot - generate snapshot versions for testing */
interface BumpSnapshotParams {
  /** Workspace root directory path */
  root?: string;
  /** Optional custom config file path */
  configPath?: string;
  /** Filter to specific packages */
  packages?: string[];
  
  /**
   * Snapshot format template.
   * Variables: {version}, {branch}, {short_commit}, {commit}, {timestamp}
   * Default: "{version}-snapshot.{short_commit}"
   * Example: "{version}-{branch}.{short_commit}" → "1.2.3-feature-x.abc123f"
   */
  format?: string;
}
```

#### Respostas (TypeScript)

```typescript
/** Package version information */
interface PackageVersionInfo {
  /** Package name (may include scope) */
  name: string;
  /** Package path relative to workspace root */
  path: string;
  /** Current version before bump */
  currentVersion: string;
  /** Next version after bump */
  nextVersion: string;
  /** Bump type applied */
  bump: 'major' | 'minor' | 'patch' | 'none';
  /** Dependency updates for this package */
  dependencyUpdates: DependencyUpdate[];
}

/** Dependency update information */
interface DependencyUpdate {
  /** Dependency name */
  name: string;
  /** Dependency type */
  type: 'regular' | 'dev' | 'peer' | 'optional';
  /** Old version spec */
  oldVersion: string;
  /** New version spec */
  newVersion: string;
}

/** Response data for bumpPreview */
interface BumpPreviewData {
  /** Version strategy used */
  strategy: 'independent' | 'unified';
  /** Packages that will be bumped */
  packages: PackageVersionInfo[];
  /** Summary of changes */
  summary: {
    totalPackages: number;
    majorBumps: number;
    minorBumps: number;
    patchBumps: number;
  };
  /** Changesets that will be consumed */
  changesets: string[];
}

/** Response data for bumpApply */
interface BumpApplyData {
  /** Version strategy used */
  strategy: 'independent' | 'unified';
  /** Packages that were bumped */
  packagesUpdated: number;
  /** Changesets that were archived */
  changesetsArchived: number;
  /** Files that were modified */
  filesModified: string[];
  /** Git tags that were created */
  tagsCreated: string[];
  /** Git commit SHA (if gitCommit was true) */
  commitSha?: string;
}

/** Snapshot version information */
interface SnapshotVersionInfo {
  /** Package name */
  name: string;
  /** Package path */
  path: string;
  /** Original version */
  originalVersion: string;
  /** Generated snapshot version */
  snapshotVersion: string;
}

/** Response data for bumpSnapshot */
interface BumpSnapshotData {
  /** Version strategy used */
  strategy: 'independent' | 'unified';
  /** Packages with snapshot versions */
  packages: SnapshotVersionInfo[];
  /** Format template used */
  format: string;
}
```

#### Implementação Rust

```rust
use crate::error::{cli_error_to_info, ErrorInfo};
use crate::response::ApiResponse;
use crate::types::bump::{
    BumpPreviewParams, BumpPreviewData, BumpApplyParams, BumpApplyData,
    BumpSnapshotParams, BumpSnapshotData,
};
use crate::validation::validators;
use napi_derive::napi;

/// Preview version bumps without applying changes.
///
/// Shows what versions would change based on pending changesets.
/// No files are modified.
///
/// @param params - Preview parameters
/// @returns Promise<ApiResponse<BumpPreviewData>> - Preview data or error
#[napi]
pub async fn bump_preview(params: BumpPreviewParams) -> ApiResponse<BumpPreviewData> {
    // 1. Validate root path if provided
    if let Some(ref root) = params.root {
        if let Err(e) = validators::path_exists(root) {
            return ApiResponse::failure(e.into());
        }
    }

    // 2. Create BumpArgs with dry_run = true
    let args = BumpArgs {
        dry_run: true,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: params.packages.clone(),
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: params.show_diff.unwrap_or(false),
    };

    // 3. Execute CLI command and parse response
    // ... implementation
    todo!()
}

/// Apply version bumps to packages.
///
/// Applies version changes based on pending changesets.
/// Optionally creates Git commits and tags.
///
/// @param params - Apply parameters
/// @returns Promise<ApiResponse<BumpApplyData>> - Apply result or error
#[napi]
pub async fn bump_apply(params: BumpApplyParams) -> ApiResponse<BumpApplyData> {
    // 1. Validate root path if provided
    if let Some(ref root) = params.root {
        if let Err(e) = validators::path_exists(root) {
            return ApiResponse::failure(e.into());
        }
    }

    // 2. Validate prerelease tag if provided
    if let Some(ref tag) = params.prerelease {
        if let Err(e) = validators::prerelease_tag(tag) {
            return ApiResponse::failure(e.into());
        }
    }

    // 3. Create BumpArgs with execute = true
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: params.prerelease.clone(),
        packages: params.packages.clone(),
        git_tag: params.git_tag.unwrap_or(false),
        git_push: params.git_push.unwrap_or(false),
        git_commit: params.git_commit.unwrap_or(false),
        no_changelog: params.no_changelog.unwrap_or(false),
        no_archive: params.no_archive.unwrap_or(false),
        always_archive: params.always_archive.unwrap_or(false),
        force: params.force.unwrap_or(true), // API is non-interactive
        show_diff: false,
    };

    // 4. Execute CLI command and parse response
    // ... implementation
    todo!()
}

/// Generate snapshot versions for testing.
///
/// Creates temporary, non-persisted versions for branch builds
/// and preview deployments. Does NOT consume changesets.
///
/// @param params - Snapshot parameters
/// @returns Promise<ApiResponse<BumpSnapshotData>> - Snapshot versions or error
#[napi]
pub async fn bump_snapshot(params: BumpSnapshotParams) -> ApiResponse<BumpSnapshotData> {
    // 1. Validate root path if provided
    if let Some(ref root) = params.root {
        if let Err(e) = validators::path_exists(root) {
            return ApiResponse::failure(e.into());
        }
    }

    // 2. Validate snapshot format if provided
    if let Some(ref format) = params.format {
        if let Err(e) = validators::snapshot_format(format) {
            return ApiResponse::failure(e.into());
        }
    }

    // 3. Create BumpArgs with snapshot = true
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: true,
        snapshot_format: params.format.clone(),
        prerelease: None,
        packages: params.packages.clone(),
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        always_archive: false,
        force: true,
        show_diff: false,
    };

    // 4. Execute CLI command and parse response
    // ... implementation
    todo!()
}
```

#### Validação Adicional

```rust
pub mod validators {
    use crate::validation::ValidationError;

    /// Validates a prerelease tag.
    /// 
    /// Valid tags contain only ASCII alphanumerics and hyphens [0-9A-Za-z-].
    pub fn prerelease_tag(tag: &str) -> Result<(), ValidationError> {
        if tag.is_empty() {
            return Err(ValidationError::required("prerelease"));
        }
        
        // Check for valid characters (SemVer 2.0.0 spec)
        if !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ValidationError::invalid(
                "prerelease",
                tag,
                "must contain only ASCII alphanumerics and hyphens [0-9A-Za-z-]",
            ));
        }
        
        Ok(())
    }

    /// Validates a snapshot format template.
    /// 
    /// Valid variables: {version}, {branch}, {short_commit}, {commit}, {timestamp}
    pub fn snapshot_format(format: &str) -> Result<(), ValidationError> {
        if format.is_empty() {
            return Err(ValidationError::required("format"));
        }
        
        // Check for at least one valid variable
        let valid_vars = ["{version}", "{branch}", "{short_commit}", "{commit}", "{timestamp}"];
        let has_valid_var = valid_vars.iter().any(|v| format.contains(v));
        
        if !has_valid_var {
            return Err(ValidationError::invalid(
                "format",
                format,
                "must contain at least one variable: {version}, {branch}, {short_commit}, {commit}, {timestamp}",
            ));
        }
        
        Ok(())
    }
}
```

#### Exemplos de Uso

```typescript
import { bumpPreview, bumpApply, bumpSnapshot } from '@websublime/workspace-tools';

// 1. Preview what will change
const preview = await bumpPreview({ 
    root: '/path/to/project',
    showDiff: true 
});

if (preview.success) {
    console.log(`Will bump ${preview.data.packages.length} packages`);
    for (const pkg of preview.data.packages) {
        console.log(`  ${pkg.name}: ${pkg.currentVersion} → ${pkg.nextVersion} (${pkg.bump})`);
    }
}

// 2. Apply changes with Git integration
const apply = await bumpApply({
    root: '/path/to/project',
    gitCommit: true,
    gitTag: true,
    gitPush: true,
});

if (apply.success) {
    console.log(`Updated ${apply.data.packagesUpdated} packages`);
    console.log(`Commit: ${apply.data.commitSha}`);
}

// 3. Create prerelease version
const prerelease = await bumpApply({
    root: '/path/to/project',
    prerelease: 'beta',
    gitCommit: true,
    gitTag: true,
});

// 4. Generate snapshot versions for CI
const snapshot = await bumpSnapshot({
    root: '/path/to/project',
    format: '{version}-{branch}.{short_commit}',
});

if (snapshot.success) {
    for (const pkg of snapshot.data.packages) {
        console.log(`${pkg.name}: ${pkg.snapshotVersion}`);
        // Output: @org/core: 1.2.3-feature-x.abc123f
    }
}
```

| Função | Params Struct | Response Struct | Pattern |
|--------|---------------|-----------------|---------|
| `init` | `InitParams` | `InitData` | Legacy |
| `configShow` | `ConfigShowParams` | `ConfigData` | Legacy |
| `configValidate` | `ConfigValidateParams` | `ValidationResult` | Legacy |
| `changesetAdd` | `ChangesetAddParams` | `ChangesetData` | Moderno |
| `changesetUpdate` | `ChangesetUpdateParams` | `ChangesetData` | Moderno |
| `changesetList` | `ChangesetListParams` | `ChangesetListData` | Moderno |
| `changesetShow` | `ChangesetShowParams` | `ChangesetData` | Moderno |
| `changesetRemove` | `ChangesetRemoveParams` | `RemoveResult` | Moderno |
| `changesetHistory` | `ChangesetHistoryParams` | `HistoryData` | Moderno |
| `changesetCheck` | `ChangesetCheckParams` | `CheckResult` | Moderno |
| `bumpPreview` | `BumpPreviewParams` | `BumpPreviewData` | Moderno |
| `bumpApply` | `BumpApplyParams` | `BumpApplyData` | Moderno |
| `bumpSnapshot` | `BumpSnapshotParams` | `BumpSnapshotData` | Moderno |
| `upgradeCheck` | `UpgradeCheckParams` | `UpgradeCheckData` | Moderno |
| `upgradeApply` | `UpgradeApplyParams` | `UpgradeApplyData` | Moderno |
| `backupList` | `BackupListParams` | `BackupListData` | Moderno |
| `backupRestore` | `BackupRestoreParams` | `RestoreResult` | Moderno |
| `backupClean` | `BackupCleanParams` | `CleanResult` | Moderno |
| `audit` | `AuditParams` | `AuditData` | Moderno |
| `changes` | `ChangesParams` | `ChangesData` | Moderno |
| `clone` | `CloneParams` | `CloneData` | Legacy |
| `status` | `StatusParams` | `StatusData` | Moderno |
| `execute` | `ExecuteParams` | `ExecuteData` | Moderno |

---

## 12. PADRÕES DE CÓDIGO E IMPLEMENTAÇÃO

### 12.1 lib.rs Entry Point

```rust
//! Node.js bindings for Sublime Workspace CLI Tools.
//!
//! This crate provides native Node.js bindings using napi-rs
//! for all CLI execute functions.

#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod error;
mod validation;
mod response;
mod types;
mod commands;

// Re-export all command functions
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
pub use commands::status::status;
pub use commands::execute::execute;

// Re-export types for TypeScript generation
pub use error::ErrorInfo;
pub use response::ApiResponse;
pub use types::*;
```

### 12.2 Padrão de Implementação de Comando

Cada comando segue este padrão:

```rust
use napi_derive::napi;
use std::io::Cursor;
use std::path::PathBuf;

use crate::error::{cli_error_to_info, ErrorInfo};
use crate::response::ApiResponse;
use crate::types::command::{CommandData, CommandParams};
use crate::validation;

/// Validates command parameters.
fn validate_params(params: &CommandParams) -> Result<(), ErrorInfo> {
    // Validation logic...
    Ok(())
}

/// Converts CLI response to NAPI types.
fn parse_response(json: &str) -> Result<CommandData, ErrorInfo> {
    // Parse JSON and convert...
    todo!()
}

/// Command description for JSDoc.
///
/// Detailed explanation of what the command does.
#[napi]
pub async fn command_name(params: CommandParams) -> napi::Result<ApiResponse<CommandData>> {
    // 1. Validate parameters
    if let Err(error) = validate_params(&params) {
        return Ok(ApiResponse::failure(error));
    }
    
    // 2. Prepare paths and args
    let root = PathBuf::from(&params.root);
    let args = build_cli_args(&params);
    
    // 3. Execute with JSON capture
    let mut buffer = Vec::new();
    {
        let output = Output::new(OutputFormat::Json, Cursor::new(&mut buffer), true);
        
        if let Err(e) = execute_cli_command(&args, &output, &root).await {
            return Ok(ApiResponse::failure(cli_error_to_info(&e)));
        }
    }
    
    // 4. Parse and return
    let json_str = String::from_utf8(buffer)
        .map_err(|e| napi::Error::from_reason(format!("UTF-8 error: {e}")))?;
    
    match parse_response(&json_str) {
        Ok(data) => Ok(ApiResponse::success(data)),
        Err(e) => Ok(ApiResponse::failure(e)),
    }
}
```

---

## 13. TESTING STRATEGY

### 13.1 Rust Unit Tests

**Localização**: `crates/node/src/**/*.rs`  
**Coverage target**: 100%

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_code_mapping() {
        let cli_err = CliError::Git("Repository not found".to_string());
        let info = cli_error_to_info(&cli_err);
        assert_eq!(info.code, "EGIT");
        assert_eq!(info.kind, "Git");
    }
    
    #[test]
    fn test_validation_required_field() {
        let err = ValidationError::required("root");
        let info: ErrorInfo = err.into();
        assert_eq!(info.code, "EVALIDATION");
        assert!(info.message.contains("required"));
    }
    
    #[test]
    fn test_timeout_validation() {
        assert!(validators::timeout("test", Some(60)).is_ok());
        assert!(validators::timeout("test", Some(0)).is_err());
        assert!(validators::timeout("test", Some(4000)).is_err());
    }
}
```

### 13.2 Node.js Integration Tests

**Localização**: `packages/workspace-tools/__test__/`  
**Coverage target**: >90%

```typescript
import test from 'ava';
import { status, execute } from '../src/index.js';

test('status - returns workspace info with proper types', async (t) => {
    const result = await status({
        root: process.cwd()
    });
    
    t.true(result.success);
    t.truthy(result.data);
    t.truthy(result.data.repository);
    t.truthy(result.data.packageManager);
    t.truthy(result.data.packages);
    t.is(result.error, undefined);
});

test('status - returns error for invalid root', async (t) => {
    const result = await status({
        root: '/non/existent/path'
    });
    
    t.false(result.success);
    t.is(result.data, undefined);
    t.truthy(result.error);
    t.is(result.error.code, 'ENOENT');
});

test('execute - validates mutual exclusion', async (t) => {
    const result = await execute({
        root: process.cwd(),
        cmd: 'npm:test',
        filterPackage: ['@scope/core'],
        affected: true  // Cannot use with filterPackage
    });
    
    t.false(result.success);
    t.is(result.error.code, 'EVALIDATION');
});

test('execute - runs command with timeout', async (t) => {
    const result = await execute({
        root: process.cwd(),
        cmd: 'echo hello',
        timeoutSecs: 30
    });
    
    t.true(result.success);
    t.truthy(result.data.summary);
});
```

---

## 14. BUILD, DISTRIBUTION & CI/CD

### 14.1 Build Commands

```bash
# Build bindings (debug)
cd packages/workspace-tools
pnpm build-binding

# Build bindings (release)
pnpm build-binding:release

# Run tests
pnpm test
```

### 14.2 GitHub Actions Workflow

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

## 15. EXEMPLOS DE USO COMPLETOS

### 15.1 Release Workflow

```typescript
import { 
    status,
    changesetList, 
    bumpPreview, 
    bumpApply 
} from '@websublime/workspace-tools';

async function releaseWorkflow() {
    const root = process.cwd();
    
    // 0. Check workspace status
    const statusResult = await status({ root });
    
    if (!statusResult.success) {
        console.error(`Error: [${statusResult.error.code}] ${statusResult.error.message}`);
        process.exit(1);
    }
    
    const { repository, packageManager, branch, packages } = statusResult.data;
    
    console.log(`Repository: ${repository.kind}`);
    console.log(`Package Manager: ${packageManager.name}`);
    console.log(`Branch: ${branch?.name || 'unknown'}`);
    console.log(`Packages: ${packages.length}`);
    
    // 1. List changesets
    const listResult = await changesetList({ root });
    
    if (!listResult.success) {
        console.error(`Error: ${listResult.error.message}`);
        process.exit(1);
    }
    
    if (listResult.data.changesets.length === 0) {
        console.log('No changesets found');
        return;
    }
    
    // 2. Preview bumps
    const previewResult = await bumpPreview({ root, showDiff: true });
    
    if (!previewResult.success) {
        console.error(`Error: ${previewResult.error.message}`);
        process.exit(1);
    }
    
    console.log('Packages to bump:');
    for (const pkg of previewResult.data.packages) {
        console.log(`  ${pkg.name}: ${pkg.currentVersion} → ${pkg.newVersion}`);
    }
    
    // 3. Apply bumps
    const applyResult = await bumpApply({
        root,
        execute: true,
        gitCommit: true,
        gitTag: true
    });
    
    if (applyResult.success) {
        console.log('✅ Version bumps applied');
    } else {
        console.error(`❌ Failed: ${applyResult.error.message}`);
        process.exit(1);
    }
}

releaseWorkflow();
```

### 15.2 Execute Commands with Timeout

```typescript
import { execute } from '@websublime/workspace-tools';

async function runTestsOnAffected() {
    const result = await execute({
        root: process.cwd(),
        cmd: 'npm:test',
        affected: true,
        branch: 'main',
        parallel: true,
        timeoutSecs: 300,           // 5 min total
        perPackageTimeoutSecs: 60   // 1 min per package
    });
    
    if (!result.success) {
        const { code, message } = result.error;
        
        if (code === 'ETIMEOUT') {
            console.error('Tests timed out!');
        } else {
            console.error(`[${code}] ${message}`);
        }
        process.exit(1);
    }
    
    const { command, results, summary } = result.data;
    
    console.log(`Command: ${command}`);
    console.log(`Total: ${summary.total}`);
    console.log(`Succeeded: ${summary.succeeded}`);
    console.log(`Failed: ${summary.failed}`);
    console.log(`Duration: ${summary.totalDurationMs}ms`);
    
    for (const pkg of results) {
        const icon = pkg.success ? '✅' : '❌';
        console.log(`${icon} ${pkg.package}: ${pkg.durationMs}ms`);
        if (pkg.error) {
            console.log(`   Error: ${pkg.error}`);
        }
    }
    
    if (summary.failed > 0) {
        process.exit(1);
    }
}

runTestsOnAffected();
```

### 15.3 Error Handling Pattern

```typescript
import { status, ApiResponse, StatusData, ErrorInfo } from '@websublime/workspace-tools';

async function handleWithErrors() {
    const result: ApiResponse<StatusData> = await status({ 
        root: '/path/to/project' 
    });
    
    if (!result.success) {
        handleError(result.error);
        return;
    }
    
    // TypeScript knows result.data is StatusData here
    console.log(result.data.packages);
}

function handleError(error: ErrorInfo): void {
    switch (error.code) {
        case 'ENOENT':
            console.error(`Path not found: ${error.message}`);
            break;
        case 'EVALIDATION':
            console.error(`Invalid parameters: ${error.message}`);
            if (error.context) {
                console.error(`  Field: ${error.context}`);
            }
            break;
        case 'EGIT':
            console.error(`Git error: ${error.message}`);
            break;
        case 'ECONFIG':
            console.error(`Configuration error: ${error.message}`);
            break;
        case 'ETIMEOUT':
            console.error(`Operation timed out: ${error.message}`);
            break;
        default:
            console.error(`[${error.code}] ${error.message}`);
    }
}
```

---

## 16. PERFORMANCE & OPTIMIZATION

### 16.1 Expected Performance

| Operation | CLI Process | NAPI Binding | Speedup |
|-----------|-------------|--------------|---------|
| `init` | ~200ms | ~50ms | 4x |
| `changesetAdd` | ~150ms | ~30ms | 5x |
| `bumpPreview` | ~500ms | ~100ms | 5x |
| `status` | ~100ms | ~20ms | 5x |
| `execute` (per pkg) | ~50ms overhead | ~5ms overhead | 10x |

**Key advantages:**
- Eliminação do process spawn overhead (~50-100ms)
- Objectos JS nativos (sem JSON.parse)
- Validação imediata na camada NAPI

---

## 17. ROADMAP DE IMPLEMENTAÇÃO

### Phase 1: Foundation (Semana 1)
- [ ] Criar estrutura `crates/node/`
- [ ] Configurar Cargo.toml
- [ ] Implementar error.rs (ErrorInfo, códigos)
- [ ] Implementar validation.rs
- [ ] Implementar response.rs (ApiResponse)
- [ ] Setup CI/CD básico

### Phase 2: POC (Semana 2)
- [ ] Implementar status (mais simples)
- [ ] Implementar init
- [ ] Testes Node.js
- [ ] Verificar TypeScript defs gerados

### Phase 3: Changeset (Semana 3)
- [ ] Implementar 7 funções changeset
- [ ] Testes completos
- [ ] Exemplos

### Phase 4: Bump (Semana 4)
- [ ] Implementar 3 funções bump
- [ ] Testes
- [ ] Workflow examples

### Phase 5: Upgrade (Semana 4-5)
- [ ] Implementar 5 funções upgrade
- [ ] Testes
- [ ] Documentation

### Phase 6: Execute & Config (Semana 5)
- [ ] Implementar execute com timeout
- [ ] Adicionar ExecuteConfig ao repo.config
- [ ] Implementar configShow, configValidate
- [ ] Testes parallel execution

### Phase 7: Remaining (Semana 5-6)
- [ ] Implementar audit, changes, clone
- [ ] Testes
- [ ] Examples

### Phase 8: Polish (Semana 6-7)
- [ ] Code review
- [ ] 100% test coverage
- [ ] Documentation completa
- [ ] Performance benchmarks

### Phase 9: Release (Semana 7-8)
- [ ] GitHub release
- [ ] npm publish
- [ ] Monitor feedback

**TOTAL**: 7-8 semanas

---

## 18. RISCOS, MITIGAÇÕES E DECISÕES

### 18.1 Riscos Técnicos

#### Risco 1: Conversão de Types CLI → NAPI
**Mitigação**: Módulo dedicado para parsing de JSON do CLI e conversão para structs NAPI

#### Risco 2: Breaking Changes no CLI
**Mitigação**: Mesmo repo = mudanças atómicas, testes detectam incompatibilidades

#### Risco 3: Platform-Specific Issues
**Mitigação**: CI testa todas plataformas, napi-rs é maduro

#### Risco 4: Execute Command Timeouts
**Mitigação**: Config + parâmetro com valores sensatos por default

### 18.2 Decisões Arquiteturais

#### Objectos JS Nativos vs JSON String
**Escolha**: Objectos JS nativos via `#[napi(object)]`  
**Rationale**: Type-safe, melhor DX, sem JSON.parse

#### Códigos de Erro
**Escolha**: Códigos estilo Node.js (EVALIDATION, EGIT, etc.)  
**Rationale**: Familiar para developers Node.js, fácil de fazer switch/case

#### Validação
**Escolha**: Validar na camada NAPI antes de chamar CLI  
**Rationale**: Fail fast, mensagens mais claras

#### Timeout para Execute
**Escolha**: Config (default) + parâmetro (override)  
**Rationale**: Flexibilidade máxima, bons defaults

#### Novo Crate vs Módulo
**Escolha**: Novo crate `crates/node/`  
**Rationale**: Separação clara, builds independentes

---

## 19. APÊNDICES

### Apêndice A: Códigos de Erro Completos

| Código | Descrição | CliError | HTTP Equiv |
|--------|-----------|----------|------------|
| `ECONFIG` | Erro de configuração | Configuration | 500 |
| `EVALIDATION` | Parâmetros inválidos | Validation | 400 |
| `EEXEC` | Erro de execução | Execution | 500 |
| `EGIT` | Erro Git | Git | 500 |
| `EPKG` | Erro de package | Package | 400 |
| `ENOENT` | Ficheiro não encontrado | Io | 404 |
| `EIO` | Erro de I/O | Io | 500 |
| `ENETWORK` | Erro de rede | Network | 503 |
| `EUSER` | Cancelado | User | 499 |
| `ETIMEOUT` | Timeout | Novo | 504 |

### Apêndice B: Comandos Úteis

```bash
# Build
pnpm build-binding:release

# Test
pnpm test

# Clippy
cargo clippy --all-targets -p sublime_node_tools

# Coverage
cargo tarpaulin --out Html -p sublime_node_tools

# Publish
npm publish --access public
```

### Apêndice C: Checklist de Release

- [ ] Clippy 100%
- [ ] Tests 100% (Rust)
- [ ] Tests >90% (Node.js)
- [ ] Documentation completa
- [ ] CHANGELOG atualizado
- [ ] TypeScript types verificados
- [ ] Build todas plataformas
- [ ] npm publish

### Apêndice D: Configuração Execute (repo.config)

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

### Apêndice E: Parâmetros Removidos na API

| Parâmetro CLI | Valor Fixo na API | Justificação |
|---------------|-------------------|--------------|
| `--non-interactive` | `true` | APIs não devem ter prompts |
| `--format` | N/A (objectos JS) | Output estruturado nativo |
| `--no-color` | `true` | Sem terminal |

---

## CONCLUSÃO

Este relatório documenta uma estratégia **completa e implementável** para criar Node.js bindings usando napi-rs para 23 funções execute do CLI.

### Benefícios Finais

✅ **Reuso 100%**: Zero duplicação de lógica  
✅ **Type-safe**: Objectos JS nativos com TypeScript  
✅ **Error Codes**: Padrão Node.js (EVALIDATION, EGIT, etc.)  
✅ **Validação**: Fail fast na camada NAPI  
✅ **Cross-platform**: macOS, Linux, Windows  
✅ **Performance**: 4-10x speedup vs CLI  
✅ **Timeout**: Config + override por parâmetro  
✅ **DX**: Sem JSON.parse, tipos completos  

### Estimativas

- **Tempo**: 7-8 semanas
- **LOC**: ~5000-6000
- **Funções**: 23
- **Error Codes**: 10
- **Complexidade**: Média
- **Risco**: Baixo

---

**Fim do Relatório**