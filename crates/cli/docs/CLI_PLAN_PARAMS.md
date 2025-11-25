# Plano de Melhorias CLI - Consistência de Parâmetros

## Contexto

Este plano aborda todas as inconsistências e problemas identificados na avaliação dos comandos CLI, organizados por prioridade e impacto.

---

## Fase 1: Correções Críticas (Breaking UX)

### 1.1 Adicionar alias `verbose` ao LogLevel

**Ficheiros a modificar:**
- `crates/cli/src/cli/args.rs`

**Alterações:**
```rust
// Em FromStr para LogLevel, adicionar:
impl FromStr for LogLevel {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "silent" => Ok(Self::Silent),
            "error" => Ok(Self::Error),
            "warn" | "warning" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" | "verbose" => Ok(Self::Debug),  // ADICIONAR "verbose"
            "trace" => Ok(Self::Trace),
            _ => Err(format!(
                "Invalid log level '{s}'. Valid options: silent, error, warn, info, debug, verbose, trace"
            )),
        }
    }
}
```

**Testes a adicionar:**
- Teste que `verbose` é aceite e mapeia para `Debug`
- Teste de help text actualizado

**Impacto:** Baixo risco, retrocompatível

---

### 1.2 Tornar `--snapshot` e `--prerelease` mutuamente exclusivos

**Contexto - Diferença semântica:**
- **Snapshot**: Versões temporárias, on-the-fly, para testing (não são escritas/persistidas)
  - Exemplo: `1.2.3-snapshot.abc123f`
  - Uso: branch builds, preview deployments, canary testing
  - Não são semver-compliant (a biblioteca semver não aceita combinações complexas)
  
- **Prerelease**: Versões oficiais de pré-lançamento, persistidas
  - Exemplo: `1.3.0-beta.0`, `2.0.0-rc.1`
  - Uso: merge para development/staging antes de main
  - São semver-compliant (alpha < beta < rc < stable)

**Limitação técnica:** A biblioteca `semver` não aceita formatos como `1.2.3-beta.snapshot.abc123f`.

**Ficheiros a modificar:**
- `crates/cli/src/cli/commands.rs`

**Alterações:**
```rust
// Em BumpArgs:
/// Generate snapshot versions for testing.
///
/// Creates temporary, non-persisted versions for branch builds and preview deployments.
/// Format: `{version}-snapshot.{short_commit}` (configurable via --snapshot-format)
/// Cannot be combined with --prerelease.
#[arg(long, conflicts_with = "prerelease")]
pub snapshot: bool,

/// Pre-release tag for official pre-release versions.
///
/// Creates semver-compliant pre-release versions (alpha, beta, rc).
/// These are persisted versions for staging/development releases.
/// Cannot be combined with --snapshot.
///
/// Options: alpha, beta, rc (or custom tag)
#[arg(long, value_name = "TAG", conflicts_with = "snapshot")]
pub prerelease: Option<String>,
```

**Testes a adicionar:**
- Teste que `--snapshot --prerelease alpha` dá erro de conflito
- Teste que `--snapshot` sozinho funciona
- Teste que `--prerelease beta` sozinho funciona

**Impacto:** Baixo - adiciona validação que previne uso inválido

---

### 1.3 Adicionar flag global `--quiet` / `-q`

**Ficheiros a modificar:**
- `crates/cli/src/cli/mod.rs`
- `crates/cli/src/cli/dispatch.rs` (para processar a flag)

**Alterações em mod.rs:**
```rust
pub struct Cli {
    // ... campos existentes ...

    /// Quiet mode.
    ///
    /// Equivalent to `--log-level silent --format quiet`.
    /// Minimizes both logs (stderr) and output (stdout).
    /// Useful for scripts and automation.
    #[arg(global = true, short = 'q', long)]
    pub quiet: bool,
}

impl Cli {
    /// Returns the effective log level, accounting for --quiet flag.
    #[must_use]
    pub fn effective_log_level(&self) -> LogLevel {
        if self.quiet {
            LogLevel::Silent
        } else {
            self.log_level
        }
    }

    /// Returns the effective output format, accounting for --quiet flag.
    #[must_use]
    pub fn effective_output_format(&self) -> OutputFormat {
        if self.quiet {
            OutputFormat::Quiet
        } else {
            self.format.0
        }
    }
}
```

**Alterações em dispatch.rs:**
- Usar `cli.effective_log_level()` em vez de `cli.log_level`
- Usar `cli.effective_output_format()` em vez de `cli.format.0`

**Testes a adicionar:**
- Teste que `-q` define log_level=silent e format=quiet
- Teste que `--quiet` funciona igual
- Teste que `--quiet --log-level debug` dá erro de conflito

---

## Fase 2: Melhorias de Consistência

### 2.1 Simplificar padrão `--major/--no-major`

**Ficheiros a modificar:**
- `crates/cli/src/cli/commands.rs`
- `crates/cli/src/commands/upgrade/check.rs` (adaptar lógica)

**Alterações em commands.rs:**
```rust
// ANTES (problemático):
#[arg(long, default_value = "true")]
pub major: bool,
#[arg(long, conflicts_with = "major")]
pub no_major: bool,

// DEPOIS (mais claro):
/// Exclude major version upgrades from results.
///
/// By default, major upgrades are included. Use this flag to exclude them.
#[arg(long)]
pub no_major: bool,

/// Exclude minor version upgrades from results.
#[arg(long)]
pub no_minor: bool,

/// Exclude patch version upgrades from results.
#[arg(long)]
pub no_patch: bool,
```

**Lógica em check.rs:**
```rust
let include_major = !args.no_major;
let include_minor = !args.no_minor;
let include_patch = !args.no_patch;
```

**Testes a actualizar:**
- Remover testes de `--major=true/false`
- Adicionar testes para `--no-major`, `--no-minor`, `--no-patch`

**Impacto:** BREAKING CHANGE - Remove flags `--major`, `--minor`, `--patch`

---

### 2.2 Uniformizar nomenclatura de filtros

**Ficheiros a modificar:**
- `crates/cli/src/cli/commands.rs`
- `crates/cli/src/commands/changeset/history.rs`

**Opção A - Usar prefixo `filter-` em todo o lado:**
```rust
// Em ChangesetHistoryArgs, renomear:
--package     -> --filter-package
--bump        -> --filter-bump  
--env         -> --filter-env
```

**Opção B - Remover prefixo `filter-` (mais limpo):**
```rust
// Em ChangesetListArgs, renomear:
--filter-package  -> --package
--filter-bump     -> --bump
--filter-env      -> --env
```

**Decisão necessária:** Confirmar qual padrão preferido (A ou B).

**Recomendação:** Opção B é mais limpa e standard.

**Impacto:** BREAKING CHANGE nos nomes das flags

---

### 2.3 Validar `--registry` uniformemente

**Ficheiros a modificar:**
- `crates/cli/src/cli/commands.rs` (adicionar validação)
- Ou criar função de validação partilhada

**Alterações:**
```rust
// Criar validador partilhado em commands.rs ou args.rs:
fn validate_registry_url(url: &str) -> Result<String, String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(url.to_string())
    } else {
        Err(format!(
            "Registry URL must start with http:// or https://, got: {url}"
        ))
    }
}

// Aplicar a todos os comandos com --registry:
// - InitArgs
// - UpgradeCheckArgs  
// - CloneArgs
```

---

## Fase 3: Melhorias de Usabilidade

### 3.1 Adicionar `--verbose` global (stdout)

**Contexto:** `--log-level` controla stderr. Não há forma standard de aumentar detalhe em stdout.

**Ficheiros a modificar:**
- `crates/cli/src/cli/mod.rs`
- `crates/cli/src/output/mod.rs` (adaptar Output struct)
- Vários comandos para respeitar a flag

**Alterações em mod.rs:**
```rust
pub struct Cli {
    // ... campos existentes ...

    /// Verbose output.
    ///
    /// Increases detail level in command output (stdout).
    /// Different from --log-level which controls operational logs (stderr).
    #[arg(global = true, short = 'v', long, conflicts_with = "quiet")]
    pub verbose: bool,
}
```

**Impacto:** 
- `version --verbose` já existe - verificar conflito com global
- Solução: Remover `--verbose` de VersionArgs e usar global

---

### 3.2 Documentar padrão de argumentos posicionais

**Ficheiros a modificar:**
- `crates/cli/src/cli/commands.rs` (melhorar doc comments)

**Padrão a documentar:**
1. Argumentos posicionais obrigatórios: `BRANCH` em `changeset show`, `changeset delete`
2. Argumentos posicionais opcionais com default: `BRANCH` em `changeset edit` (default: current branch)
3. Argumentos híbridos: `URL [DESTINATION]` em `clone`

**Não é breaking change** - apenas documentação.

---

### 3.3 Clarificar `--non-interactive` vs `--force`

**Ficheiros a modificar:**
- `crates/cli/src/cli/commands.rs` (melhorar doc comments)

**Semântica a documentar:**
- `--non-interactive`: Não fazer prompts interactivos, usar defaults/flags fornecidos
- `--force`: Confirmar automaticamente acções potencialmente destrutivas

**Alguns comandos podem beneficiar de ter ambos:**
- `changeset create`: Adicionar `--force` para sobrescrever changeset existente?
- `bump`: Adicionar `--non-interactive` para automação CI/CD?

**Decisão necessária:** Quais comandos devem ter ambas as flags?

---

## Fase 4: Testes e Documentação

### 4.1 Actualizar testes existentes

Para cada alteração acima, actualizar:
- `crates/cli/src/cli/tests.rs`
- Testes de integração em `crates/cli/tests/`

### 4.2 Actualizar documentação

**Ficheiros a modificar:**
- `crates/cli/README.md` (se existir)
- Help text de todos os comandos modificados
- `CHANGELOG.md` - documentar breaking changes

---

## Resumo de Breaking Changes

| Alteração | Tipo | Mitigação |
|-----------|------|-----------|
| Remover `--major`, `--minor`, `--patch` em `upgrade check` | Remoção de flags | Usar `--no-major`, `--no-minor`, `--no-patch` |
| Renomear filtros em `changeset history` | Rename de flags | `--package` → `--filter-package`, etc. |
| `--snapshot` e `--prerelease` mutuamente exclusivos | Nova validação | Usar apenas um de cada vez |

---

## Ordem de Implementação Sugerida

1. **Fase 1.1** - Alias `verbose` (sem breaking changes)
2. **Fase 1.3** - Flag `--quiet` (sem breaking changes)
3. **Fase 1.2** - Conflito snapshot/prerelease (minor breaking)
4. **Fase 2.3** - Validação registry (sem breaking changes)
5. **Fase 2.1** - Simplificar major/minor/patch (BREAKING)
6. **Fase 2.2** - Uniformizar filtros (BREAKING)
7. **Fase 3.x** - Melhorias de usabilidade
8. **Fase 4** - Testes e documentação

---

## Decisões Tomadas

As seguintes decisões foram confirmadas:

1. ✅ **Snapshot + Prerelease**: Mutuamente exclusivos (são conceitos diferentes)
   - Snapshot: versões temporárias para testing
   - Prerelease: versões oficiais de pré-lançamento (semver-compliant)

2. ✅ **Filtros**: Usar padrão `--filter-*` em `changeset history` para consistência com `changeset list`

3. ✅ **Breaking Changes**: Implementar todas numa release major (sem deprecation warnings)

4. ✅ **`--verbose` global**: Implementar agora

5. ✅ **`--non-interactive` / `--force`**: Adicionar a comandos em falta

6. ✅ **Compatibilidade**: Não há necessidade de manter backwards compatibility
