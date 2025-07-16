# Plano de Consolidação do Crate sublime_package_tools

## Objetivo
Limpar o crate `sublime_package_tools` removendo código de compatibilidade desnecessário e reorganizando a estrutura para maior clareza, sem quebrar funcionalidade existente.

## Fase 1: Remoção de Type Aliases de Compatibilidade

### 1.1 Aliases a Remover
- [ ] `DependencyChange` → usar `Change` diretamente
- [ ] `DependencyFilter` → usar `Filter` diretamente  
- [ ] `DependencyUpdate` → usar `Update` diretamente
- [ ] `DependencyRegistry` → usar `Registry` diretamente
- [ ] `DependencyGraph` → usar `Graph` diretamente
- [ ] `PackageInfo` → usar `Info` diretamente

### 1.2 Ações por Ficheiro

#### dependency/change.rs
- [ ] Remover linha 168: `pub type DependencyChange = Change;`
- [ ] Remover comentário de compatibilidade

#### dependency/filter.rs
- [ ] Remover linha 66: `pub type DependencyFilter = Filter;`
- [ ] Remover comentário de compatibilidade

#### dependency/update.rs
- [ ] Remover linha 62: `pub type DependencyUpdate = Update;`
- [ ] Remover comentário de compatibilidade

#### dependency/registry.rs
- [ ] Remover linha 670: `pub type DependencyRegistry = Registry;`
- [ ] Remover comentário de compatibilidade

#### dependency/graph.rs
- [ ] Remover linha 1075: `pub type DependencyGraph<'a, N> = Graph<'a, N>;`
- [ ] Remover comentário de compatibilidade

#### package/info.rs
- [ ] Remover linha 409: `pub type PackageInfo = Info;`
- [ ] Remover comentário de compatibilidade

#### lib.rs
- [ ] Atualizar linha 182-189: exportar `Info` em vez de `PackageInfo`
- [ ] Atualizar linha 191-195: exportar `Change`, `Filter`, `Graph`, `Registry`, `Update` diretamente

## Fase 2: Reorganização dos Módulos de Grafo

### 2.1 Decisão Estrutural
Optamos por **clarificar responsabilidades** mantendo:
- `dependency/graph.rs` → implementação específica para grafos de dependências
- `graph/` → utilitários genéricos (builder, validation, visualization)

### 2.2 Ações
- [ ] Adicionar documentação clara em `dependency/graph.rs` explicando sua responsabilidade
- [ ] Adicionar documentação clara em `graph/mod.rs` explicando a separação
- [ ] Verificar e ajustar imports internos se necessário

## Fase 3: Limpeza de Código Deprecated

### 3.1 Método deprecated em dependency/dependency.rs
- [ ] Remover método `set_version` (linha ~329)
- [ ] Remover comentário de compatibilidade
- [ ] Garantir que todos os usos internos usam `with_version`

### 3.2 Erro não utilizado em errors/dependency.rs
- [ ] Remover variante `IncompatibleVersions` do enum (linha ~15)
- [ ] Remover comentário de compatibilidade

## Fase 4: Atualização da Documentação

### 4.1 SPEC.md
- [ ] Atualizar todas as referências aos aliases antigos
- [ ] Usar nomes diretos: `Change`, `Filter`, `Update`, `Registry`, `Graph`, `Info`

### 4.2 README.md
- [ ] Verificar e atualizar exemplos se necessário

### 4.3 Documentação inline
- [ ] Atualizar exemplos de código nos comentários de documentação

## Fase 5: Validação Final

### 5.1 Testes
- [ ] Executar `cargo test` no crate pkg
- [ ] Executar `cargo test` no crate monorepo
- [ ] Verificar que todos os testes passam

### 5.2 Compilação
- [ ] `cargo build` sem erros
- [ ] `cargo clippy -- -D warnings` sem avisos

### 5.3 Documentação
- [ ] `cargo doc --no-deps` gera documentação correta

## Ordem de Execução Recomendada

1. **Primeiro**: Fase 1 (Remoção de aliases) - impacto direto mas simples
2. **Segundo**: Fase 3 (Limpeza de código deprecated) - remove código não utilizado
3. **Terceiro**: Fase 2 (Reorganização) - melhoria estrutural sem quebrar API
4. **Quarto**: Fase 4 (Documentação) - atualizar para refletir mudanças
5. **Último**: Fase 5 (Validação) - garantir que tudo funciona

## Riscos e Mitigações

### Risco 1: Código externo usando os aliases
- **Mitigação**: Análise mostrou que monorepo não usa aliases
- **Ação**: Verificar se há outros consumidores externos antes de publicar

### Risco 2: Quebra de testes
- **Mitigação**: Executar testes após cada fase
- **Ação**: Corrigir testes incrementalmente

## Benefícios Esperados

1. **API mais clara**: Nomes diretos sem prefixos redundantes
2. **Menos código**: Remoção de ~12 linhas de aliases + comentários
3. **Manutenção simplificada**: Menos indireção no código
4. **Documentação melhorada**: Estrutura mais clara e intuitiva

## Notas Importantes

- Não há necessidade de manter compatibilidade segundo as instruções
- O crate monorepo já está preparado para as mudanças
- Todas as mudanças são breaking changes mas aceitáveis em desenvolvimento