# CLI Pattern Refactoring Plan

**Objetivo**: Uniformizar todas as funções `execute_*` do CLI para usar o mesmo padrão de assinatura (Pattern B - Moderno)

**Status**: Planning  
**Impacto**: Breaking changes internos (não afeta API do CLI binário)  
**Estimativa**: 3-5 dias  

---

## EXECUTIVE SUMMARY

### Problema Identificado

O CLI possui **duas assinaturas diferentes** para funções execute, criando inconsistência e complexidade:

**Pattern A (Legacy)** - 4 funções:
```rust
async fn execute_*(
    args: &ArgsStruct,
    root: &Path,
    format: OutputFormat  // ❌ Enum direto, sem flexibilidade
) -> Result<()>
```

**Pattern B (Moderno)** - 23 funções:
```rust
async fn execute_*(
    args: &ArgsStruct,
    output: &Output,  // ✅ Struct com writer, format, no_color
    root: &Path ou Option<&Path>,
    config_path: Option<&Path>  // ✅ Config path separado
) -> Result<()>
```

### Impacto na Codebase

| Tipo de Impacto | Pattern A | Pattern B | Problema |
|-----------------|-----------|-----------|----------|
| **Output Flexibility** | ❌ Apenas enum | ✅ Struct com writer | Não podemos capturar output |
| **Config Path** | ❌ Hardcoded | ✅ Opcional | Menos flexibilidade |
| **Root Path** | ❌ Sempre `&Path` | ✅ `Option<&Path>` | Menos flexibilidade |
| **Testabilidade** | ❌ Difícil mock | ✅ Fácil mock | Testes mais complexos |
| **Node.js Bindings** | ❌ **BLOCKER** | ✅ Funciona | Não conseguimos capturar JSON |

### Decisão

✅ **Migrar TODAS as funções para Pattern B** (sem backward compatibility)

**Rationale:**
1. **Consistência**: Um único padrão, código mais limpo
2. **Flexibilidade**: Output struct permite testes, capture, mocking
3. **Node.js Bindings**: Desbloqueia implementação (podemos capturar JSON)
4. **Manutenibilidade**: Menos código duplicado, padrão único
5. **Breaking é aceitável**: APIs internas, não afeta usuários do CLI

---

## 1. ANÁLISE DETALHADA

### 1.1 Funções Pattern A (Legacy) - 4 funções

| # | Função | Arquivo | Linha | Usado em |
|---|--------|---------|-------|----------|
| 1 | `execute_init` | `commands/init.rs` | 88 | dispatch.rs:104, e2e_init.rs |
| 2 | `execute_show` (config) | `commands/config.rs` | 94 | dispatch.rs:108, e2e_config.rs |
| 3 | `execute_validate` (config) | `commands/config.rs` | 210 | dispatch.rs:112, e2e_config.rs |
| 4 | `execute_clone` | `commands/clone.rs` | 798 | dispatch.rs:276, e2e_clone.rs |

### 1.2 Funções Pattern B (Moderno) - 23 funções

Todas as outras funções já seguem Pattern B:
- **Changeset**: add, update, list, show, edit, remove, history, check (8)
- **Bump**: preview, apply, snapshot (3)
- **Upgrade**: check, apply, backup_list, backup_restore, backup_clean (5)
- **Audit**: comprehensive, upgrades, dependencies, versions, breaking (5)
- **Changes**: execute_changes (1)
- **Version**: execute_version (1) - NOTA: Esta é sync, não async!

### 1.3 Código Duplicado em Pattern A

#### execute_init - output_init_result()

```rust
// init.rs linha ~200
fn output_init_result(
    config_path: &Path,
    config: &InitConfig,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Human => {
            println!("\n{}", Style::success("✓ Workspace initialized successfully!"));
            // ... mais prints
        }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let response = JsonResponse::success(InitResult { /* ... */ });
            let json = if format == OutputFormat::Json {
                serde_json::to_string_pretty(&response)?
            } else {
                serde_json::to_string(&response)?
            };
            println!("{json}");
        }
        OutputFormat::Quiet => {
            // Silencioso
        }
    }
    Ok(())
}
```

**Problema**: Duplica lógica que já existe em `Output::success()`, `Output::json()`, etc.

#### execute_show/validate - output_*_format()

```rust
// config.rs
fn output_human_format(config: &PackageToolsConfig, is_default: bool) {
    println!("\n{}", Style::header("Workspace Configuration"));
    // ... prints manuais
}

fn output_json_format(config: &PackageToolsConfig, format: OutputFormat) -> Result<()> {
    let response = JsonResponse::success(config);
    let json = if format == OutputFormat::Json {
        serde_json::to_string_pretty(&response)?
    } else {
        serde_json::to_string(&response)?
    };
    println!("{json}");
    Ok(())
}
```

**Problema**: Mesma duplicação! Pattern B usa `output.json(&response)` que já faz isso.

#### execute_clone - múltiplas funções output_*

```rust
// clone.rs
fn output_clone_complete(destination: &Path, validated: bool, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Human => { /* ... */ }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let response = JsonResponse::success(/* ... */);
            let json = if format == OutputFormat::Json {
                serde_json::to_string_pretty(&response)?
            } else {
                serde_json::to_string(&response)?
            };
            println!("{json}");
        }
        // ...
    }
    Ok(())
}
```

**Problema**: Novamente, código que `Output` já fornece!

### 1.4 Problemas para Node.js Bindings

**Pattern A bloqueia implementação de bindings:**

```rust
// ❌ Pattern A - Não conseguimos capturar output!
pub async fn init(params: InitParams) -> napi::Result<String> {
    execute_init(&args, root, OutputFormat::Json).await?;
    // ❌ Output foi para stdout! Não capturamos nada!
    // Só podemos retornar JSON estático genérico
    Ok(r#"{"success": true}"#.to_string())
}

// ✅ Pattern B - Conseguimos capturar!
pub async fn changeset_add(params: Params) -> napi::Result<String> {
    let mut buffer = Vec::new();
    let output = Output::new(OutputFormat::Json, Cursor::new(&mut buffer), true);
    
    execute_add(&args, &output, root, config_path).await?;
    
    // ✅ Buffer tem o JSON completo!
    String::from_utf8(buffer).map_err(...)
}
```

---

## 2. PLANO DE REFACTORING

### 2.1 Mudanças de Assinatura

#### ANTES (Pattern A):
```rust
// init.rs
pub async fn execute_init(
    args: &InitArgs,
    root: &Path,
    format: OutputFormat
) -> Result<()>

// config.rs
pub async fn execute_show(
    args: &ConfigShowArgs,
    root: &Path,
    config_path: Option<&Path>,
    format: OutputFormat
) -> Result<()>

pub async fn execute_validate(
    args: &ConfigValidateArgs,
    root: &Path,
    config_path: Option<&Path>,
    format: OutputFormat
) -> Result<()>

// clone.rs
pub async fn execute_clone(
    args: &CloneArgs,
    root: &Path,
    config_path: Option<&Path>,
    format: OutputFormat
) -> Result<()>
```

#### DEPOIS (Pattern B Uniformizado):
```rust
// init.rs
pub async fn execute_init(
    args: &InitArgs,
    output: &Output,  // ← NOVO
    root: &Path,
    config_path: Option<&Path>  // ← NOVO
) -> Result<()>

// config.rs
pub async fn execute_show(
    args: &ConfigShowArgs,
    output: &Output,  // ← MUDOU
    root: &Path,
    config_path: Option<&Path>
) -> Result<()>

pub async fn execute_validate(
    args: &ConfigValidateArgs,
    output: &Output,  // ← MUDOU
    root: &Path,
    config_path: Option<&Path>
) -> Result<()>

// clone.rs
pub async fn execute_clone(
    args: &CloneArgs,
    output: &Output,  // ← MUDOU
    root: &Path,
    config_path: Option<&Path>
) -> Result<()>
```

### 2.2 Refactoring Interno das Funções

#### execute_init

**ANTES:**
```rust
pub async fn execute_init(args: &InitArgs, root: &Path, format: OutputFormat) -> Result<()> {
    // ... lógica ...
    
    // Output result
    output_init_result(&config_file_path, &init_config, format)?;  // ❌ Função custom
    
    Ok(())
}

fn output_init_result(
    config_path: &Path,
    config: &InitConfig,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Human => {
            println!("\n{}", Style::success("✓ Workspace initialized successfully!"));
            println!("\nConfiguration:");
            println!("  File: {}", Style::path(config_path));
            // ... mais prints
        }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let response = JsonResponse::success(InitResult {
                config_file: config_path.display().to_string(),
                changeset_path: config.changeset_path.clone(),
                environments: config.environments.clone(),
                strategy: config.strategy.clone(),
            });
            let json = if format == OutputFormat::Json {
                serde_json::to_string_pretty(&response)?
            } else {
                serde_json::to_string(&response)?
            };
            println!("{json}");
        }
        OutputFormat::Quiet => {
            // Silent
        }
    }
    Ok(())
}
```

**DEPOIS:**
```rust
pub async fn execute_init(
    args: &InitArgs,
    output: &Output,  // ← Novo parâmetro
    root: &Path,
    config_path: Option<&Path>  // ← Novo parâmetro
) -> Result<()> {
    // ... lógica (mesma) ...
    
    // Output result usando Output struct
    output_init_result(&config_file_path, &init_config, output)?;  // ← output em vez de format
    
    Ok(())
}

fn output_init_result(
    config_path: &Path,
    config: &InitConfig,
    output: &Output,  // ← Mudou de OutputFormat para &Output
) -> Result<()> {
    match output.format() {  // ← output.format() em vez de format
        OutputFormat::Human => {
            output.success("Workspace initialized successfully!")?;  // ← Usa output.success()
            output.info("Configuration:")?;
            output.info(&format!("  File: {}", config_path.display()))?;
            output.info(&format!("  Changeset path: {}", config.changeset_path))?;
            output.info(&format!("  Strategy: {}", config.strategy))?;
            
            if !config.environments.is_empty() {
                output.info(&format!("  Environments: {}", config.environments.join(", ")))?;
            }
            
            output.blank_line()?;
            output.info("Next steps:")?;
            output.info("  1. Create a feature branch")?;
            output.info("  2. Make your changes")?;
            output.info("  3. Run: workspace changeset create")?;
        }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let result = InitResult {
                config_file: config_path.display().to_string(),
                changeset_path: config.changeset_path.clone(),
                environments: config.environments.clone(),
                strategy: config.strategy.clone(),
            };
            let response = JsonResponse::success(result);
            output.json(&response)?;  // ← Usa output.json() que já lida com pretty/compact
        }
        OutputFormat::Quiet => {
            // Silent - output já lida com quiet mode
        }
    }
    Ok(())
}
```

**Benefícios:**
- ✅ Remove ~50 linhas de código duplicado
- ✅ Usa métodos `Output` existentes
- ✅ Permite capturar output para testes/bindings
- ✅ Consistente com outras funções

#### execute_show (config)

**ANTES:**
```rust
pub async fn execute_show(
    _args: &ConfigShowArgs,
    root: &Path,
    config_path: Option<&Path>,
    format: OutputFormat,
) -> Result<()> {
    // ... load config ...
    
    match format {
        OutputFormat::Human => output_human_format(&config, is_default),
        OutputFormat::Json | OutputFormat::JsonCompact => output_json_format(&config, format)?,
        OutputFormat::Quiet => output_quiet_format(&config),
    }
    
    Ok(())
}

fn output_human_format(config: &PackageToolsConfig, is_default: bool) {
    println!("\n{}", Style::header("Workspace Configuration"));
    if is_default {
        println!("{}", Style::warning("⚠ Using default configuration (no config file found)"));
    }
    println!("\nChangeset Settings:");
    println!("  Path: {}", Style::value(&config.changeset_path));
    // ... mais 30 linhas de prints
}
```

**DEPOIS:**
```rust
pub async fn execute_show(
    _args: &ConfigShowArgs,
    output: &Output,  // ← Mudou
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    // ... load config (mesmo código) ...
    
    match output.format() {
        OutputFormat::Human => {
            output.header("Workspace Configuration")?;
            
            if is_default {
                output.warning("Using default configuration (no config file found)")?;
            }
            
            output.blank_line()?;
            output.section("Changeset Settings")?;
            output.field("Path", &config.changeset_path)?;
            output.field("Formats", &config.formats.join(", "))?;
            
            output.section("Version Strategy")?;
            output.field("Strategy", &config.strategy.to_string())?;
            
            // ... etc, usando métodos de Output
        }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let response = JsonResponse::success(&config);
            output.json(&response)?;
        }
        OutputFormat::Quiet => {
            output.quiet(&format!("Strategy: {}", config.strategy))?;
        }
    }
    
    Ok(())
}
```

**Benefícios:**
- ✅ Remove 3 funções auxiliares (~80 linhas)
- ✅ Usa `output.header()`, `output.field()`, etc. (métodos padronizados)
- ✅ Consistente com changeset commands

#### execute_validate (config)

Similar ao `execute_show`, mas com validação:

**DEPOIS:**
```rust
pub async fn execute_validate(
    _args: &ConfigValidateArgs,
    output: &Output,  // ← Mudou
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    // ... load and validate config ...
    
    match output.format() {
        OutputFormat::Human => {
            output.header("Configuration Validation")?;
            
            for check in &validation_results.checks {
                if check.passed {
                    output.success(&format!("✓ {}", check.description))?;
                } else {
                    output.error(&format!("✗ {}: {}", check.description, check.error_message.as_ref().unwrap()))?;
                }
            }
            
            if validation_results.is_valid() {
                output.blank_line()?;
                output.success("Configuration is valid!")?;
            } else {
                return Err(CliError::validation("Configuration validation failed"));
            }
        }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let response = JsonResponse::success(&validation_results);
            output.json(&response)?;
        }
        OutputFormat::Quiet => {
            if validation_results.is_valid() {
                output.quiet("valid")?;
            } else {
                return Err(CliError::validation("invalid"));
            }
        }
    }
    
    Ok(())
}
```

#### execute_clone

Mais complexo porque tem múltiplas chamadas de output em diferentes fases:

**ANTES:**
```rust
pub async fn execute_clone(
    args: &CloneArgs,
    root: &Path,
    config_path: Option<&Path>,
    format: OutputFormat,
) -> Result<()> {
    // ... clone repo ...
    
    if let Some(_config) = config_opt {
        output_validation_starting(format);  // ❌ Custom function
        // ... validate ...
        output_validation_success(&validation, format);  // ❌ Custom function
        output_clone_complete(&final_destination, validated, format)?;  // ❌ Custom function
    } else {
        output_init_starting(format);  // ❌ Custom function
        execute_init(&init_args, &final_destination, format).await?;  // ❌ Pattern A!
        output_clone_complete_with_init(&final_destination, format)?;  // ❌ Custom function
    }
    
    Ok(())
}
```

**DEPOIS:**
```rust
pub async fn execute_clone(
    args: &CloneArgs,
    output: &Output,  // ← Mudou
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    // ... clone repo (mesmo código) ...
    
    if let Some(_config) = config_opt {
        if !args.skip_validation {
            output.info("Validating workspace configuration...")?;  // ← Usa output.info()
            
            let validation = validate_workspace(&final_destination).await?;
            
            if !validation.is_valid {
                return Err(CliError::validation(format!(
                    "Workspace configuration validation failed:\n\n{validation}"
                )));
            }
            
            output.success("Workspace configuration validated successfully")?;  // ← output.success()
        }
        
        // Output completion
        match output.format() {
            OutputFormat::Human => {
                output.success(&format!("Repository cloned to: {}", final_destination.display()))?;
                if validated {
                    output.info("Workspace configuration is valid")?;
                }
            }
            OutputFormat::Json | OutputFormat::JsonCompact => {
                let result = CloneResult {
                    destination: final_destination.display().to_string(),
                    validated,
                    initialized: false,
                };
                output.json(&JsonResponse::success(result))?;
            }
            OutputFormat::Quiet => {}
        }
    } else {
        output.info("No workspace configuration found, initializing...")?;  // ← output.info()
        
        let init_args = convert_to_init_args(args, None);
        
        // ✅ Agora execute_init também usa Pattern B!
        execute_init(&init_args, output, &final_destination, None).await?;
        
        match output.format() {
            OutputFormat::Human => {
                output.success(&format!("Repository cloned and initialized at: {}", final_destination.display()))?;
            }
            OutputFormat::Json | OutputFormat::JsonCompact => {
                let result = CloneResult {
                    destination: final_destination.display().to_string(),
                    validated: false,
                    initialized: true,
                };
                output.json(&JsonResponse::success(result))?;
            }
            OutputFormat::Quiet => {}
        }
    }
    
    Ok(())
}
```

**Benefícios:**
- ✅ Remove 5 funções auxiliares de output (~150 linhas)
- ✅ Consistente com outros comandos
- ✅ execute_init agora usa Pattern B (pode capturar output se precisar)

---

## 3. IMPACTO EM dispatch.rs

### 3.1 Mudanças Necessárias

**ANTES:**
```rust
// dispatch.rs linha 87+
pub async fn dispatch_command(cli: &Cli) -> Result<()> {
    let root = cli.root.as_deref().unwrap_or_else(|| Path::new("."));
    let format = cli.output_format();
    let config_path = cli.config_path();

    match &cli.command {
        Commands::Init(args) => {
            init::execute_init(args, root, format).await?;  // ❌ Pattern A
        }

        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Show(args) => {
                config::execute_show(args, root, config_path.map(PathBuf::as_path), format).await?;  // ❌ Pattern A
            }
            ConfigCommands::Validate(args) => {
                config::execute_validate(args, root, config_path.map(PathBuf::as_path), format).await?;  // ❌ Pattern A
            }
        },
        
        // ... changesets já usam Pattern B corretamente ...

        Commands::Clone(args) => {
            clone::execute_clone(args, root, config_path.map(PathBuf::as_path), format).await?;  // ❌ Pattern A
        }
    }

    Ok(())
}
```

**DEPOIS:**
```rust
pub async fn dispatch_command(cli: &Cli) -> Result<()> {
    let root = cli.root.as_deref().unwrap_or_else(|| Path::new("."));
    let format = cli.output_format();
    let config_path = cli.config_path();

    match &cli.command {
        Commands::Init(args) => {
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());  // ✅ Criar Output
            init::execute_init(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;  // ✅ Pattern B
        }

        Commands::Config(config_cmd) => {
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());  // ✅ Criar Output
            match config_cmd {
                ConfigCommands::Show(args) => {
                    config::execute_show(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;  // ✅ Pattern B
                }
                ConfigCommands::Validate(args) => {
                    config::execute_validate(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;  // ✅ Pattern B
                }
            }
        },
        
        // ... changesets continuam iguais (já estão corretos) ...

        Commands::Clone(args) => {
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());  // ✅ Criar Output
            clone::execute_clone(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;  // ✅ Pattern B
        }
    }

    Ok(())
}
```

**Mudanças:**
1. Criar `Output` antes de chamar funções Pattern A (4 locais)
2. Passar `&output` em vez de `format`
3. Passar `config_path` onde necessário (init não tinha antes)

### 3.2 Otimização Opcional: Criar Output Uma Vez

Podemos criar `Output` uma vez no início:

```rust
pub async fn dispatch_command(cli: &Cli) -> Result<()> {
    let root = cli.root.as_deref().unwrap_or_else(|| Path::new("."));
    let format = cli.output_format();
    let config_path = cli.config_path();
    
    // ✅ Criar output UMA vez
    let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());

    match &cli.command {
        Commands::Init(args) => {
            init::execute_init(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;
        }

        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Show(args) => {
                config::execute_show(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;
            }
            ConfigCommands::Validate(args) => {
                config::execute_validate(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;
            }
        },
        
        Commands::Changeset(changeset_cmd) => {
            // ✅ Já não precisa criar output aqui!
            match changeset_cmd {
                ChangesetCommands::Create(args) => {
                    changeset::execute_add(args, &output, /* ... */).await?;
                }
                // ... etc
            }
        }

        Commands::Bump(args) => {
            // ✅ Já não precisa criar output aqui!
            if args.snapshot {
                bump::execute_bump_snapshot(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;
            } else if args.execute {
                bump::execute_bump_apply(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;
            } else {
                bump::execute_bump_preview(args, &output, root, config_path.as_ref().map(|p| p.as_path())).await?;
            }
        }

        // ... etc, todos usam &output
    }

    Ok(())
}
```

**Benefício adicional**: Remove ~15 linhas de código duplicado em dispatch.rs!

---

## 4. IMPACTO EM TESTES

### 4.1 Testes Afetados

| Arquivo | Funções Testadas | Impacto |
|---------|-----------------|---------|
| `e2e_init.rs` | `execute_init` | ⚠️ Médio |
| `e2e_config.rs` | `execute_show`, `execute_validate` | ⚠️ Médio |
| `e2e_clone.rs` | `execute_clone` | ⚠️ Alto |
| `common/helpers.rs` | Pode ter helpers | ⚠️ Baixo |

### 4.2 Padrão de Atualização de Testes

**ANTES:**
```rust
// e2e_init.rs
#[tokio::test]
async fn test_init_creates_config() {
    let temp_dir = create_test_workspace();
    
    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        // ...
        non_interactive: true,
    };
    
    // ❌ Pattern A
    execute_init(&args, temp_dir.path(), OutputFormat::Json).await.unwrap();
    
    assert!(temp_dir.path().join("repo.config.json").exists());
}
```

**DEPOIS:**
```rust
#[tokio::test]
async fn test_init_creates_config() {
    let temp_dir = create_test_workspace();
    
    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        // ...
        non_interactive: true,
    };
    
    // ✅ Pattern B - Criar Output com buffer para capturar
    let mut buffer = Vec::new();
    let output = Output::new(
        OutputFormat::Json,
        Cursor::new(&mut buffer),
        true
    );
    
    execute_init(&args, &output, temp_dir.path(), None).await.unwrap();
    
    // ✅ Agora podemos verificar o output também!
    let json_output = String::from_utf8(buffer).unwrap();
    assert!(json_output.contains("success"));
    
    // ✅ E verificar filesystem
    assert!(temp_dir.path().join("repo.config.json").exists());
}
```

**Benefícios dos testes após refactoring:**
- ✅ Podemos capturar e verificar output
- ✅ Testes mais completos (verificam behavior + output)
- ✅ Testam em modo JSON (garantindo JSON válido)

### 4.3 Helper Utilities para Testes

Criar helper em `common/helpers.rs`:

```rust
// common/helpers.rs

/// Cria um Output para testes que captura para buffer
pub fn create_test_output(format: OutputFormat) -> (Output, Vec<u8>) {
    let buffer = Vec::new();
    let output = Output::new(format, Cursor::new(&buffer), true);
    (output, buffer)
}

/// Executa comando e retorna output como string
pub async fn execute_with_capture<F, Fut>(
    format: OutputFormat,
    f: F
) -> Result<String>
where
    F: FnOnce(Output) -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let mut buffer = Vec::new();
    let output = Output::new(format, Cursor::new(&mut buffer), true);
    
    f(output).await?;
    
    String::from_utf8(buffer)
        .map_err(|e| CliError::execution(format!("Invalid UTF-8: {e}")))
}
```

**Uso:**
```rust
#[tokio::test]
async fn test_init_json_output() {
    let temp_dir = create_test_workspace();
    let args = /* ... */;
    
    let json = execute_with_capture(OutputFormat::Json, |output| async {
        execute_init(&args, &output, temp_dir.path(), None).await
    }).await.unwrap();
    
    let response: JsonResponse<InitResult> = serde_json::from_str(&json).unwrap();
    assert!(response.success);
    assert_eq!(response.data.unwrap().strategy, "independent");
}
```

---

## 5. CHECKLIST DE IMPLEMENTAÇÃO

### Phase 1: Preparação (1 dia)

- [ ] **Criar branch**: `refactor/cli-uniform-pattern`
- [ ] **Backup**: Criar tag `pre-pattern-refactor`
- [ ] **Documentar**: Este plano (DONE ✅)
- [ ] **Comunicar**: Avisar equipa sobre breaking changes internos

### Phase 2: execute_init (0.5 dia)

- [ ] **Atualizar assinatura** em `commands/init.rs:88`
  - Adicionar parâmetro `output: &Output`
  - Adicionar parâmetro `config_path: Option<&Path>`
  
- [ ] **Refatorar output_init_result()** 
  - Mudar de `OutputFormat` para `&Output`
  - Substituir `println!()` por `output.success()`, `output.info()`, etc.
  - Substituir JSON manual por `output.json()`
  
- [ ] **Remover funções duplicadas**
  - Remover código de formatting manual
  
- [ ] **Atualizar dispatch.rs**
  - Criar `Output` antes de chamar
  - Passar `&output` e `config_path`
  
- [ ] **Atualizar e2e_init.rs**
  - Atualizar todos os testes
  - Adicionar verificação de output capturado
  
- [ ] **Compilar e testar**: `cargo test -p sublime_cli_tools init`

### Phase 3: execute_show e execute_validate (0.5 dia)

- [ ] **Atualizar assinaturas** em `commands/config.rs`
  - `execute_show`: Mudar `format: OutputFormat` → `output: &Output`
  - `execute_validate`: Mudar `format: OutputFormat` → `output: &Output`
  
- [ ] **Refatorar output functions**
  - `output_human_format()` → usar `output.header()`, `output.field()`, etc.
  - `output_json_format()` → usar `output.json()`
  - `output_quiet_format()` → usar `output.quiet()`
  - Remover funções auxiliares
  
- [ ] **Atualizar dispatch.rs**
  - Passar `&output` em vez de `format`
  
- [ ] **Atualizar e2e_config.rs**
  - Atualizar todos os testes
  - Adicionar verificação de output
  
- [ ] **Compilar e testar**: `cargo test -p sublime_cli_tools config`

### Phase 4: execute_clone (1 dia)

- [ ] **Atualizar assinatura** em `commands/clone.rs:798`
  - Mudar `format: OutputFormat` → `output: &Output`
  
- [ ] **Refatorar todas as output functions**
  - `output_clone_complete()` → usar `output.*`
  - `output_clone_complete_with_init()` → usar `output.*`
  - `output_validation_starting()` → `output.info()`
  - `output_validation_success()` → `output.success()`
  - `output_init_starting()` → `output.info()`
  - Remover todas as 5+ funções auxiliares
  
- [ ] **Atualizar chamada para execute_init**
  - Passar `output` em vez de `format`
  - execute_init já estará atualizado (Phase 2)
  
- [ ] **Atualizar dispatch.rs**
  - Passar `&output`
  
- [ ] **Atualizar e2e_clone.rs**
  - Atualizar todos os testes
  - Adicionar verificação de output
  
- [ ] **Compilar e testar**: `cargo test -p sublime_cli_tools clone`

### Phase 5: Otimização dispatch.rs (0.5 dia)

- [ ] **Criar Output uma vez** no início de `dispatch_command()`
- [ ] **Remover criações duplicadas** de Output em cada branch
- [ ] **Simplificar código** (~15 linhas a menos)
- [ ] **Testar**: `cargo test -p sublime_cli_tools`

### Phase 6: Helpers de Teste (0.5 dia)

- [ ] **Criar helpers** em `common/helpers.rs`
  - `create_test_output()`
  - `execute_with_capture()`
  
- [ ] **Refatorar testes existentes** para usar helpers
  - Menos boilerplate
  - Mais consistente
  
- [ ] **Testar**: `cargo test -p sublime_cli_tools`

### Phase 7: Verificação Final (1 dia)

- [ ] **Clippy**: `cargo clippy --all-targets -- -D warnings`
- [ ] **Format**: `cargo fmt --all -- --check`
- [ ] **Tests**: `cargo test --all`
- [ ] **Integration tests**: Rodar suite completa
- [ ] **Manual testing**: Testar cada comando alterado manualmente
  - `workspace init`
  - `workspace config show`
  - `workspace config validate`
  - `workspace clone <url>`

### Phase 8: Documentação e Merge (0.5 dia)

- [ ] **Atualizar CHANGELOG.md**
  - Seção "Internal Changes"
  - Listar breaking changes internos
  
- [ ] **Atualizar comentários** de módulo se necessário
- [ ] **PR**: Criar PR detalhado com:
  - Link para este plano
  - Resumo de mudanças
  - Testes rodados
  - Breaking changes (internos)
  
- [ ] **Review**: Code review
- [ ] **Merge**: Merge para main

---

## 6. MÉTRICAS DE SUCESSO

### 6.1 Código Removido

| Arquivo | Linhas Antes | Linhas Depois | Redução |
|---------|--------------|---------------|---------|
| `init.rs` | ~500 | ~450 | -50 (-10%) |
| `config.rs` | ~800 | ~650 | -150 (-19%) |
| `clone.rs` | ~1100 | ~950 | -150 (-14%) |
| `dispatch.rs` | ~280 | ~260 | -20 (-7%) |
| **TOTAL** | **~2680** | **~2310** | **-370 (-14%)** |

### 6.2 Consistência

| Métrica | Antes | Depois |
|---------|-------|--------|
| **Padrões de assinatura** | 2 | 1 ✅ |
| **Funções execute_*** | 27 | 27 |
| **Pattern A (legacy)** | 4 | 0 ✅ |
| **Pattern B (moderno)** | 23 | 27 ✅ |
| **Funções auxiliares de output** | ~15 | ~5 ✅ |

### 6.3 Testabilidade

| Aspecto | Antes | Depois |
|---------|-------|--------|
| **Output capturável** | ❌ Pattern A | ✅ Todos |
| **Mock de Output** | ❌ Difícil | ✅ Fácil |
| **Testes verificam output** | Parcial | ✅ Completo |

### 6.4 Node.js Bindings

| Aspecto | Antes | Depois |
|---------|-------|--------|
| **Funções bloqueadas** | 4 | 0 ✅ |
| **Capture JSON** | ❌ Impossível | ✅ Possível |
| **Output detalhado** | JSON estático | ✅ JSON real |

---

## 7. RISCOS E MITIGAÇÕES

### 7.1 Risco: Breaking Changes

**Probabilidade**: Alta  
**Impacto**: Baixo (interno apenas)

**Descrição**: Mudanças de assinatura quebram código que chama essas funções.

**Mitigação**:
- ✅ **Escopo limitado**: Apenas `dispatch.rs` e testes chamam essas funções
- ✅ **Detecção**: Compilador detecta todas as quebras
- ✅ **Fix simultâneo**: Atualizar chamadas no mesmo commit
- ✅ **Sem impacto externo**: API do CLI binário não muda

### 7.2 Risco: Regressões de Comportamento

**Probabilidade**: Média  
**Impacto**: Médio

**Descrição**: Refactoring pode introduzir bugs sutis em output formatting.

**Mitigação**:
- ✅ **Testes existentes**: Suite de testes cobre behavior
- ✅ **Testes novos**: Adicionar verificação de output capturado
- ✅ **Manual testing**: Testar cada comando manualmente
- ✅ **Incremental**: Fazer uma função de cada vez
- ✅ **Rollback fácil**: Branch + tag de backup

### 7.3 Risco: Output Struct Limitações

**Probabilidade**: Baixa  
**Impacto**: Baixo

**Descrição**: `Output` pode não ter métodos para todos os casos de uso.

**Mitigação**:
- ✅ **Output é extensível**: Podemos adicionar métodos se precisar
- ✅ **Análise prévia**: Já verificamos que Output tem o necessário
- ✅ **Fallback**: Podemos usar `output.write()` diretamente se precisar

### 7.4 Risco: Performance

**Probabilidade**: Muito Baixa  
**Impacto**: Muito Baixo

**Descrição**: Criar `Output` struct pode ter overhead.

**Mitigação**:
- ✅ **Overhead negligível**: Struct leve, apenas wraps writer
- ✅ **Mesma lógica**: Não muda algoritmos, só interface
- ✅ **Benchmarks**: Rodar benchmarks se suspeita de regressão

---

## 8. ALTERNATIVAS CONSIDERADAS

### 8.1 Alternativa: Manter Ambos os Padrões

**Descrição**: Manter Pattern A e B coexistindo.

**Prós**:
- ✅ Zero breaking changes
- ✅ Menos trabalho

**Contras**:
- ❌ Inconsistência permanente
- ❌ Duplicação de código
- ❌ Bloqueia Node.js bindings
- ❌ Confusão para novos contribuidores

**Decisão**: ❌ Rejeitada

### 8.2 Alternativa: Criar Wrappers v2

**Descrição**: Criar funções `execute_*_v2` com Pattern B, manter v1.

```rust
pub async fn execute_init(args: &InitArgs, root: &Path, format: OutputFormat) -> Result<()> {
    let output = Output::new(format, std::io::stdout(), false);
    execute_init_v2(args, &output, root, None).await
}

pub async fn execute_init_v2(
    args: &InitArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>
) -> Result<()> {
    // Implementação real
}
```

**Prós**:
- ✅ Backward compatible
- ✅ Permite migração gradual

**Contras**:
- ❌ Duplicação de funções
- ❌ Confusão: qual usar?
- ❌ Manutenção duplicada
- ❌ Ainda temos inconsistência

**Decisão**: ❌ Rejeitada (complexidade não justifica benefício)

### 8.3 Alternativa: Macro para Abstrair Diferença

**Descrição**: Criar macro que funciona com ambos os padrões.

```rust
macro_rules! execute {
    ($fn:expr, $args:expr, $root:expr, $format:expr) => {
        // Magic para detectar Pattern A vs B
    };
}
```

**Prós**:
- ✅ Sem breaking changes
- ✅ Abstração da diferença

**Contras**:
- ❌ **Muito complexo**
- ❌ Debugging difícil
- ❌ Não resolve problema raiz
- ❌ Over-engineering

**Decisão**: ❌ Rejeitada (complexidade inaceitável)

---

## 9. CONCLUSÃO

### 9.1 Resumo

Este refactoring:
- ✅ **Uniformiza** 27 funções execute para um único padrão
- ✅ **Remove** ~370 linhas de código duplicado
- ✅ **Desbloqueia** implementação de Node.js bindings
- ✅ **Melhora** testabilidade e consistência
- ✅ **Baixo risco**: Breaking changes apenas internos
- ✅ **Tempo estimado**: 3-5 dias

### 9.2 Recomendação

**✅ APROVAR E EXECUTAR**

**Justificativa**:
1. **Necessário para Node.js bindings**: Pattern A bloqueia captura de output
2. **Melhoria de qualidade**: Remove duplicação, aumenta consistência
3. **Baixo risco**: Apenas APIs internas, testes detectam problemas
4. **Esforço razoável**: 3-5 dias bem investidos
5. **Benefício a longo prazo**: Codebase mais limpo e maintainável

### 9.3 Próximos Passos

1. **Revisão**: Review deste plano
2. **Aprovação**: Decidir se prosseguir
3. **Execução**: Seguir checklist Phase 1-8
4. **Validação**: Testes + manual testing
5. **Merge**: Integrar para main
6. **Seguir para bindings**: Implementar Node.js bindings usando Pattern B uniformizado

---

**Plano criado por**: Claude (Sonnet 4.5)  
**Data**: 2025-01-18  
**Versão**: 1.0  
**Status**: Aguardando aprovação
