# 🎯 Plano de Reescrita - sublime_package_tools v2.0

## 📋 Requerimentos do Crate

### Objetivo Principal
Fornecer ferramentas robustas e eficientes para gestão de packages Node.js em ambientes single repository e monorepo, com foco em:

1. **Gestão de Packages**
   - Leitura/escrita de package.json
   - Validação de estrutura e dependências
   - Suporte completo a todos os protocolos de dependências

2. **Análise de Dependências**
   - Construção de grafos de dependências
   - Deteção de ciclos e conflitos
   - Classificação internal/external

3. **Version Bumping**
   - Bump strategies (major/minor/patch/custom)
   - Cascade bumping para monorepos
   - Preview mode (dry-run)

4. **Upgrades**
   - Deteção de atualizações disponíveis
   - Aplicação segura de upgrades
   - Compatibilidade checking

### Princípios de Design

1. **Simplicidade**: APIs claras e intuitivas
2. **Performance**: Otimizado para grandes monorepos
3. **Configurabilidade**: Totalmente configurável via StandardConfig
4. **Integração**: Uso máximo dos crates base (standard, git)
5. **Zero Legacy**: Sem preocupação com compatibilidade anterior

---

## 🏗️ Arquitetura Simplificada

```
sublime_package_tools/
├── core/              # Tipos fundamentais
│   ├── package.rs     # Package struct (data only)
│   ├── dependency.rs  # Dependency types
│   └── version.rs     # Version utilities
├── services/          # Lógica de negócio
│   ├── analyzer.rs    # Package analysis
│   ├── bumper.rs      # Version bumping
│   └── upgrader.rs    # Dependency upgrades
├── graph/             # Dependency graph
│   ├── builder.rs     # Graph construction
│   └── analyzer.rs    # Graph analysis
└── lib.rs            # Public API
```

---

## 📊 Fases de Implementação

### FASE 1: Limpeza e Fundação (1 semana)

#### Objetivo
Remover todo código legacy, simplificar arquitetura e estabelecer fundação sólida.

#### Tasks

**Task 1.1: Limpeza Massiva**
- [ ] Deletar módulo `storage` completamente (Registry confusion)
- [ ] Deletar módulo `external` (será recriado simplificado)
- [ ] Deletar módulo `config` (usar StandardConfig diretamente)
- [ ] Deletar módulo `network` (mover para onde for usado)
- [ ] Remover todas as APIs marcadas como deprecated no relatório

**Task 1.2: Simplificar Core Types**
- [ ] Refatorar `Package` para ser pure data struct
- [ ] Simplificar `Dependency` removendo lógica desnecessária
- [ ] Criar `Version` utilities focadas no essencial
- [ ] Remover duplicações e tipos não utilizados

**Task 1.3: Configuração via StandardConfig**
- [ ] Integrar com StandardConfig do crate standard
- [ ] Definir PackageToolsConfig minimal (max 3 structs)
- [ ] Implementar passagem de configs para standard/git crates
- [ ] Remover toda configuração via env vars diretas

**Task 1.4: Setup Testes e CI**
- [ ] Limpar testes que não agregam valor
- [ ] Configurar estrutura de testes por módulo
- [ ] Setup CI com coverage mínimo 80%
- [ ] Adicionar benchmarks básicos

---

### FASE 2: Core Functionality (1.5 semanas)

#### Objetivo
Implementar funcionalidades core com design limpo e APIs intuitivas.

#### Tasks

**Task 2.1: Package Management**
```rust
pub struct PackageManager<F: AsyncFileSystem> {
    filesystem: F,
}

impl PackageManager {
    pub async fn read_package(&self, path: &Path) -> Result<Package>;
    pub async fn write_package(&self, path: &Path, package: &Package) -> Result<()>;
    pub async fn validate_package(&self, package: &Package) -> Result<ValidationReport>;
}
```

**Task 2.2: Dependency Analysis**
```rust
pub struct DependencyAnalyzer {
    // Simples e direto
}

impl DependencyAnalyzer {
    pub fn analyze_dependencies(&self, package: &Package) -> DependencyReport;
    pub fn find_conflicts(&self, packages: &[Package]) -> Vec<Conflict>;
    pub fn classify_dependency(&self, dep: &Dependency, workspace: Option<&WorkspaceInfo>) -> DependencyClass;
}
```

**Task 2.3: Graph Construction**
- [ ] Graph builder simples e eficiente
- [ ] Suporte a single repo e monorepo
- [ ] Deteção de ciclos com contexto
- [ ] Visualização (ASCII/DOT) como utility

**Task 2.4: Integration com Standard Crate**
- [ ] Usar AsyncFileSystem para todo I/O
- [ ] Integrar ProjectDetector para contexto
- [ ] Usar CommandExecutor para npm/yarn/pnpm
- [ ] Configuração unificada

---

### FASE 3: Version Management (1 semana)

#### Objetivo
Implementar version bumping e cascade operations de forma clara e eficiente.

#### Tasks

**Task 3.1: Version Bumper**
```rust
pub struct VersionBumper<F: AsyncFileSystem> {
    filesystem: F,
}

impl VersionBumper {
    pub async fn bump_version(&self, package: &Package, strategy: BumpStrategy) -> Result<Package>;
    pub async fn cascade_bump(&self, workspace: &WorkspaceInfo, changes: &[Change]) -> Result<BumpReport>;
    pub async fn preview_bump(&self, package: &Package, strategy: BumpStrategy) -> Result<BumpPreview>;
}
```

**Task 3.2: Bump Strategies**
- [ ] Major/Minor/Patch/Custom
- [ ] Prerelease handling
- [ ] Snapshot versions
- [ ] Workspace-aware bumping

**Task 3.3: Cascade Operations**
- [ ] Detetar packages afetados
- [ ] Aplicar bumps em cascata
- [ ] Atualizar referências internas
- [ ] Rollback em caso de erro

---

### FASE 4: Upgrade Management (1 semana)

#### Objetivo
Implementar sistema de upgrades simples mas poderoso.

#### Tasks

**Task 4.1: Upgrade Detector**
```rust
pub struct UpgradeDetector {
    registry_client: RegistryClient,
}

impl UpgradeDetector {
    pub async fn check_upgrades(&self, package: &Package) -> Result<Vec<AvailableUpgrade>>;
    pub async fn check_compatibility(&self, upgrade: &AvailableUpgrade) -> Result<CompatibilityReport>;
}
```

**Task 4.2: Upgrade Aplicator**
- [ ] Aplicar upgrades com validação
- [ ] Suporte a dry-run
- [ ] Rollback automático em falhas
- [ ] Relatórios detalhados

**Task 4.3: Registry Integration**
- [ ] Cliente HTTP simples e resiliente
- [ ] Cache inteligente
- [ ] Retry com backoff
- [ ] Suporte a registries privados

---

### FASE 5: Polish e Documentação (0.5 semana)

#### Objetivo
Finalizar com documentação completa e exemplos práticos.

#### Tasks

**Task 5.1: Documentação**
- [ ] Atualizar toda documentação inline
- [ ] Criar guia de uso completo
- [ ] Exemplos para cada caso de uso
- [ ] Migration guide (do que mudou)

**Task 5.2: Exemplos**
- [ ] Exemplo single repository
- [ ] Exemplo monorepo
- [ ] Exemplo CI/CD integration
- [ ] Exemplo custom tooling

**Task 5.3: Performance**
- [ ] Benchmarks finais
- [ ] Otimizações identificadas
- [ ] Profiling em projetos reais
- [ ] Documentar limites

---

## 🗓️ Roadmap

### Sprint 1 (Semana 1)
- ✅ FASE 1 completa
- ✅ Fundação limpa estabelecida
- ✅ CI/CD configurado

### Sprint 2 (Semana 2-3)
- ✅ FASE 2 completa
- ✅ Core functionality implementada
- ✅ Integração com standard crate

### Sprint 3 (Semana 4)
- ✅ FASE 3 completa
- ✅ Version management funcional
- ✅ Cascade operations testadas

### Sprint 4 (Semana 5)
- ✅ FASE 4 completa
- ✅ Upgrade system implementado
- ✅ Registry integration estável

### Sprint 5 (Semana 5.5)
- ✅ FASE 5 completa
- ✅ Documentação finalizada
- ✅ v2.0.0 ready para release

---

## 📐 Decisões Técnicas

### 1. Sem Backwards Compatibility
- Zero preocupação com APIs antigas
- Breaking changes são esperados
- Foco em fazer certo desta vez

### 2. Integração Total com Crates Base
- AsyncFileSystem para todo I/O
- StandardConfig para configuração
- GitTools para operações git
- Reutilizar ao máximo

### 3. Simplicidade sobre Features
- Melhor fazer pouco bem feito
- APIs intuitivas e previsíveis
- Documentação como first-class citizen

### 4. Performance por Design
- Estruturas de dados eficientes
- Operações assíncronas onde faz sentido
- Caching inteligente
- Zero alocações desnecessárias

---

## 🎯 Métricas de Sucesso

1. **Simplicidade**: < 10k linhas de código total
2. **Performance**: < 100ms para analisar monorepo com 100 packages
3. **Qualidade**: > 80% test coverage
4. **Documentação**: 100% das APIs públicas documentadas
5. **Integração**: Zero duplicação com crates base

---

## 🚀 Próximos Passos Imediatos

1. [ ] Aprovar este plano
2. [ ] Começar Task 1.1 - Limpeza massiva
3. [ ] Setup branch `v2-rewrite`
4. [ ] Comunicar breaking changes

---

**Data de Início**: Imediato após aprovação  
**Data de Conclusão Estimada**: 5.5 semanas  
**Versão Target**: 2.0.0