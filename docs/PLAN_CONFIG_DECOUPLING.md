# Plano: Desacoplamento da Descoberta de Configuração

## Objectivo

Remover toda a lógica de descoberta de ficheiros de configuração (`repo.config.{toml,yml,yaml,json}`) dos crates base (`standard`, `pkg`) e centralizar essa responsabilidade exclusivamente no CLI.

## Motivação

- **Separação de responsabilidades**: CLI = orchestração/discovery, crates = lógica de negócio
- **Flexibilidade para integradores**: Utilizadores dos crates como biblioteca podem usar qualquer nome/formato de config
- **Eliminação de duplicação**: Lógica repetida 5 vezes no código actual
- **Testabilidade**: Testes não dependem de ficheiros no disco

## Dependências entre Crates

```
standard (base) ← pkg ← cli
git (base) ←────────┘
```

## Ordem de Execução

1. **Fase 1**: `standard` - remover lógica de discovery
2. **Fase 2**: `pkg` - alterar ConfigLoader para receber struct
3. **Fase 3**: `cli` - centralizar discovery e adaptar chamadas

---

## Fase 1: Crate `standard`

### Ficheiros a Modificar

| Ficheiro | Alteração |
|----------|-----------|
| `src/project/detector.rs` | Remover `load_project_config()` (linhas 385-418) e método `new_with_project_config()` |
| `src/filesystem/manager.rs` | Remover `load_project_config()` (linhas 215-240) e método `new_with_project_config()` |
| `src/monorepo/detector.rs` | Remover `load_project_config()` (linhas 487-520) e método `new_with_project_config()` |
| `src/command/executor.rs` | Remover `load_project_config()` (linhas 221-254) e método `new_with_project_config()` |
| `src/config/manager.rs` | Manter - é genérico e útil |
| `src/config/format.rs` | Manter - parsing de formatos |

### Alterações de API

**Antes:**
```rust
// Auto-discovery interno
let detector = ProjectDetector::new_with_project_config(root).await?;
```

**Depois:**
```rust
// Config explícita (ou None para defaults)
let detector = ProjectDetector::new(root, config);
```

### Detalhes de Implementação

1. **ProjectDetector** (`src/project/detector.rs`):
   - Remover método `load_project_config()` 
   - Remover `new_with_project_config()`
   - Método `new()` deve aceitar `Option<StandardConfig>`
   - Se `None`, usar `StandardConfig::default()`

2. **FileSystemManager** (`src/filesystem/manager.rs`):
   - Remover método `load_project_config()`
   - Remover `new_with_project_config()`
   - Construtor aceita config directamente

3. **MonorepoDetector** (`src/monorepo/detector.rs`):
   - Remover método `load_project_config()`
   - Remover `new_with_project_config()`
   - Construtor aceita config directamente

4. **DefaultCommandExecutor** (`src/command/executor.rs`):
   - Remover método `load_project_config()`
   - Remover `new_with_project_config()`
   - Construtor aceita config directamente

### Testes a Actualizar

- Testes que usem `*_with_project_config()` devem passar config explícita
- Remover testes de discovery interno

---

## Fase 2: Crate `pkg`

### Ficheiros a Modificar

| Ficheiro | Alteração |
|----------|-----------|
| `src/config/loader.rs` | Simplificar para receber struct já deserializada |
| `src/config/mod.rs` | Actualizar re-exports |

### Alterações de API

**Antes:**
```rust
// Carrega de ficheiro
let config = ConfigLoader::load_from_file("repo.config.toml").await?;
// Ou com discovery
let config = load_config().await?;
```

**Depois:**
```rust
// Apenas validação e defaults
let config = PackageToolsConfig::default();
// Ou construção directa
let config = PackageToolsConfig { ... };
config.validate()?;
```

### Detalhes de Implementação

1. **Remover completamente `ConfigLoader`**:
   - `load_defaults()` → usar `PackageToolsConfig::default()`
   - `load_from_file()` → responsabilidade do CLI
   - `load_from_files()` → responsabilidade do CLI
   - `load_config()` → remover (discovery)
   - `load_config_from_file()` → remover

2. **PackageToolsConfig** deve ter:
   - `Default` trait implementado (já tem)
   - Método `validate(&self) -> Result<()>` público
   - Métodos builder opcionais para configuração fluente

3. **Manter apenas**:
   - Parsing de formatos (TOML, YAML, JSON) - pode ser útil para o CLI
   - Função utilitária: `parse_config(content: &str, format: ConfigFormat) -> Result<PackageToolsConfig>`

### Nova API Proposta

```rust
// Em pkg/src/config/mod.rs
impl PackageToolsConfig {
    /// Cria configuração com valores default
    pub fn default() -> Self { ... }
    
    /// Valida a configuração
    pub fn validate(&self) -> ConfigResult<()> { ... }
    
    /// Parse de string num formato específico
    pub fn from_str(content: &str, format: ConfigFormat) -> ConfigResult<Self> { ... }
}

// Formatos suportados (manter)
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}
```

---

## Fase 3: Crate `cli`

### Ficheiros a Modificar

| Ficheiro | Alteração |
|----------|-----------|
| `src/commands/mod.rs` | Manter e melhorar `find_and_load_config()` |
| `src/commands/*.rs` | Adaptar para nova API dos crates base |

### Detalhes de Implementação

1. **Centralizar discovery** em `src/commands/mod.rs`:
   ```rust
   /// Descobre e carrega configuração
   pub async fn discover_and_load_config(
       root: &Path,
       config_path: Option<&Path>,
   ) -> Result<PackageToolsConfig> {
       let config_file = if let Some(path) = config_path {
           // Usa path explícito
           validate_config_exists(path)?;
           path.to_path_buf()
       } else {
           // Discovery: procura repo.config.{toml,json,yaml,yml}
           find_config_file(root)?
       };
       
       // Lê e faz parse
       let content = fs::read_to_string(&config_file).await?;
       let format = ConfigFormat::from_path(&config_file)?;
       let config = PackageToolsConfig::from_str(&content, format)?;
       config.validate()?;
       Ok(config)
   }
   ```

2. **Adaptar comandos** para passar config aos crates:
   ```rust
   // Exemplo em changeset/create.rs
   pub async fn execute(args: &Args, root: &Path, config: &PackageToolsConfig) {
       let detector = ProjectDetector::new(root, Some(config.clone()));
       // ...
   }
   ```

---

## Ficheiros Críticos (referência rápida)

### standard
- `crates/standard/src/project/detector.rs`
- `crates/standard/src/filesystem/manager.rs`
- `crates/standard/src/monorepo/detector.rs`
- `crates/standard/src/command/executor.rs`

### pkg
- `crates/pkg/src/config/loader.rs`
- `crates/pkg/src/config/mod.rs`
- `crates/pkg/src/config/types.rs`

### cli
- `crates/cli/src/commands/mod.rs`
- `crates/cli/src/commands/changeset/*.rs`
- `crates/cli/src/commands/bump.rs`
- `crates/cli/src/commands/config.rs`

---

## Critérios de Sucesso

- [ ] Nenhum crate base (`standard`, `pkg`) procura ficheiros `repo.config.*`
- [ ] Toda a lógica de discovery está em `cli`
- [ ] APIs públicas dos crates base recebem config já deserializada
- [ ] `cargo clippy` passa sem warnings
- [ ] Todos os testes passam
- [ ] Cada fase compila independentemente antes de avançar

## Notas

- Não há necessidade de manter compatibilidade
- Mudanças incrementais: cada fase deve compilar antes de avançar
- Documentar novas APIs conforme regras do CLAUDE.md
