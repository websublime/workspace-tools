# 🎯 FASE 2: Core Functionality - Sprint TODO

**Objetivo**: Implementar funcionalidades core com design limpo e APIs intuitivas

**Data Início**: 2025-07-28  
**Status**: ✅ Task 2.1 (Package Management) completamente concluída!  
**Progresso**: 5/20 tasks concluídas (25%)

---

## 📋 **Task 2.1: Package Management** (5/20 tasks)

### 📦 **Implementação do PackageManager**

- [x] **Task 2.1.1**: Criar estrutura base PackageManager
  - **Arquivo**: `crates/pkg/src/package/manager.rs`
  - **Struct**: `PackageManager<F: AsyncFileSystem>` com filesystem integration
  - **Traits**: Implementar AsyncFileSystem integration do crate standard
  - **Documentação**: Documentação completa com exemplos
  - **Objetivo**: Base para todas operações de package management
  - **Prioridade**: 🔴 Alta

- [x] **Task 2.1.2**: Implementar read_package method
  - **Método**: `pub async fn read_package(&self, path: &Path) -> Result<Package>`
  - **Funcionalidade**: Ler package.json e converter para Package struct
  - **Validação**: Validação de formato e campos obrigatórios
  - **Error handling**: Error types específicos para problemas de parsing
  - **Testes**: Unit tests para vários formatos de package.json
  - **Objetivo**: Leitura robusta de package.json files
  - **Prioridade**: 🔴 Alta

- [x] **Task 2.1.3**: Implementar write_package method
  - **Método**: `pub async fn write_package(&self, path: &Path, package: &Package) -> Result<()>`
  - **Funcionalidade**: Escrever Package struct para package.json
  - **Formatação**: Preservar formatação original quando possível
  - **Backup**: Criar backup antes de modificações
  - **Atomic operations**: Garantir operações atômicas (temp file + rename)
  - **Objetivo**: Escrita segura e confiável de package.json
  - **Prioridade**: 🔴 Alta

- [x] **Task 2.1.4**: Implementar validate_package method
  - **Método**: `pub async fn validate_package(&self, package: &Package) -> Result<ValidationReport>`
  - **Validações**: Nome válido, versão semver, dependencies válidas
  - **Report**: Struct ValidationReport com warnings e errors
  - **Rules**: Implementar rules comuns (semver, naming conventions)
  - **Extensibilidade**: Permitir custom validation rules
  - **Objetivo**: Validação abrangente de packages
  - **Prioridade**: 🟡 Média

- [x] **Task 2.1.5**: Integração e testes do PackageManager
  - **Testes**: Integration tests com filesystem real e mock - ✅ CONCLUÍDO
  - **Edge cases**: Arquivos corrompidos, permissões, paths inválidos - ✅ CONCLUÍDO
  - **Performance**: Benchmarks para operações de I/O - ✅ CONCLUÍDO
  - **Documentação**: Exemplos de uso completos - ✅ CONCLUÍDO
  - **Export**: Adicionar ao lib.rs e documentação pública - ✅ CONCLUÍDO
  - **Objetivo**: PackageManager production-ready - ✅ ALCANÇADO
  - **Prioridade**: 🟡 Média

---

## 🔍 **Task 2.2: Dependency Analysis** (5/20 tasks)

### 📊 **Implementação do DependencyAnalyzer**

- [ ] **Task 2.2.1**: Criar estrutura base DependencyAnalyzer
  - **Arquivo**: `crates/pkg/src/dependency/analyzer.rs`
  - **Struct**: `DependencyAnalyzer` simples e eficiente
  - **Configuration**: Integração com PackageToolsConfig
  - **Constructor**: Métodos new() e with_config()
  - **Objetivo**: Base para análise de dependências
  - **Prioridade**: 🔴 Alta

- [ ] **Task 2.2.2**: Implementar analyze_dependencies method
  - **Método**: `pub fn analyze_dependencies(&self, package: &Package) -> DependencyReport`
  - **Analysis**: Analisar production, dev, peer, optional dependencies
  - **Report**: Struct DependencyReport com estatísticas e insights
  - **Metrics**: Contar deps, detectar patterns, versioning analysis
  - **Classification**: Classificar dependencies por tipo e criticidade
  - **Objetivo**: Análise completa das dependências de um package
  - **Prioridade**: 🔴 Alta

- [ ] **Task 2.2.3**: Implementar find_conflicts method
  - **Método**: `pub fn find_conflicts(&self, packages: &[Package]) -> Vec<Conflict>`
  - **Detection**: Detectar conflicts de versão entre packages
  - **Algorithms**: Implementar algoritmos eficientes de conflict detection
  - **Report**: Struct Conflict com detalhes do conflito e sugestões
  - **Resolution**: Sugerir possíveis resoluções para conflitos
  - **Objetivo**: Identificação de conflitos entre dependencies
  - **Prioridade**: 🔴 Alta

- [ ] **Task 2.2.4**: Implementar classify_dependency method
  - **Método**: `pub fn classify_dependency(&self, dep: &Dependency, workspace: Option<&WorkspaceInfo>) -> DependencyClass`
  - **Classification**: Internal, External, Dev, Peer, Optional
  - **Workspace**: Considera contexto de workspace/monorepo
  - **Enum**: DependencyClass com todas as classificações possíveis
  - **Logic**: Lógica inteligente baseada em naming patterns e workspace
  - **Objetivo**: Classificação inteligente de dependencies
  - **Prioridade**: 🟡 Média

- [ ] **Task 2.2.5**: Integração e testes do DependencyAnalyzer
  - **Testes**: Unit tests para todos os métodos
  - **Mock data**: Criar datasets de teste realistas
  - **Performance**: Otimizar para análise de muitos packages
  - **Documentation**: Exemplos práticos de uso
  - **Export**: Integrar com lib.rs
  - **Objetivo**: DependencyAnalyzer production-ready
  - **Prioridade**: 🟡 Média

---

## 🌐 **Task 2.3: Graph Construction** (5/20 tasks)

### 🔗 **Implementação de Graph Builder e Utilities**

- [ ] **Task 2.3.1**: Criar Graph builder simples e eficiente
  - **Arquivo**: `crates/pkg/src/graph/builder.rs` (refactor do existente)
  - **Algorithms**: Implementar algoritmos eficientes de graph construction
  - **Memory**: Otimizar uso de memória para graphs grandes
  - **APIs**: APIs intuitivas para construção incremental
  - **Validation**: Validar graph integrity durante construção
  - **Objetivo**: Builder robusto e eficiente para dependency graphs
  - **Prioridade**: 🔴 Alta

- [ ] **Task 2.3.2**: Suporte a single repo e monorepo
  - **Detection**: Auto-detectar se é single repo ou monorepo
  - **Handling**: Tratamento específico para cada tipo
  - **Integration**: Integrar com ProjectDetector do crate standard
  - **Context**: Maintain context information para diferentes cenários
  - **Performance**: Otimizar para monorepos grandes
  - **Objetivo**: Suporte universal para diferentes project structures
  - **Prioridade**: 🔴 Alta

- [ ] **Task 2.3.3**: Detecção de ciclos com contexto
  - **Algorithm**: Implementar detecção eficiente de cycles
  - **Context**: Fornecer contexto detalhado sobre cycles encontrados
  - **Resolution**: Sugerir possíveis resoluções para cycles
  - **Performance**: Otimizar para graphs grandes
  - **Reporting**: Reports detalhados sobre cycles
  - **Objetivo**: Detecção inteligente de dependency cycles
  - **Prioridade**: 🟡 Média

- [ ] **Task 2.3.4**: Visualização (ASCII/DOT) como utility
  - **ASCII**: Visualização em ASCII art para terminal
  - **DOT**: Export para formato DOT (Graphviz)
  - **Filtering**: Permitir filtros para graphs grandes
  - **Styling**: Options para customizar appearance
  - **Export**: Utilities independentes e reusáveis
  - **Objetivo**: Visualização útil de dependency graphs
  - **Prioridade**: 🟢 Baixa

- [ ] **Task 2.3.5**: Otimização e testes do Graph system
  - **Performance**: Benchmarks e otimizações
  - **Memory**: Profiling de uso de memória
  - **Edge cases**: Testes com graphs complexos
  - **Integration**: Testes de integração completos
  - **Documentation**: Documentação e exemplos
  - **Objetivo**: Graph system production-ready
  - **Prioridade**: 🟡 Média

---

## 🔧 **Task 2.4: Integration com Standard Crate** (5/20 tasks)

### 🤝 **Integração Completa com Base Crates**

- [ ] **Task 2.4.1**: Usar AsyncFileSystem para todo I/O
  - **Refactor**: Substituir std::fs por AsyncFileSystem
  - **Consistency**: Garantir uso consistente em todo o codebase
  - **Testing**: Adaptar testes para usar filesystem mocks
  - **Performance**: Verificar performance das operações assíncronas
  - **Error handling**: Adaptar error handling para async operations
  - **Objetivo**: I/O completamente assíncrono e testável
  - **Prioridade**: 🔴 Alta

- [ ] **Task 2.4.2**: Integrar ProjectDetector para contexto
  - **Integration**: Usar ProjectDetector do crate standard
  - **Context**: Detect workspace type, package manager, etc.
  - **Configuration**: Adapt operations based on detected context
  - **Caching**: Cache detection results para performance
  - **Testing**: Tests com diferentes project structures
  - **Objetivo**: Context-aware operations
  - **Prioridade**: 🔴 Alta

- [ ] **Task 2.4.3**: Usar CommandExecutor para npm/yarn/pnpm
  - **Integration**: Integrar CommandExecutor do crate standard
  - **Commands**: Implementar commands para npm, yarn, pnpm
  - **Detection**: Auto-detect package manager em uso
  - **Error handling**: Proper error handling para command failures
  - **Testing**: Mock command execution para testes
  - **Objetivo**: Execução robusta de package manager commands
  - **Prioridade**: 🟡 Média

- [ ] **Task 2.4.4**: Configuração unificada
  - **Config flow**: Garantir que PackageToolsConfig flui para todos components
  - **Standard integration**: Propagate StandardConfig corretamente
  - **Validation**: Validar configuração em todos os pontos
  - **Testing**: Tests de integração com diferentes configs
  - **Documentation**: Documentar configuration flow
  - **Objetivo**: Configuração consistente em todo o sistema
  - **Prioridade**: 🟡 Média

- [ ] **Task 2.4.5**: Validação final da integração
  - **End-to-end**: Tests end-to-end com todos components integrados
  - **Performance**: Benchmarks da integração completa
  - **Documentation**: Update documentation com new integrations
  - **Examples**: Criar exemplos práticos de uso
  - **API review**: Review final das APIs públicas
  - **Objetivo**: Integração completa e validada
  - **Prioridade**: 🟡 Média

---

## 📊 **Status Summary**

### **Por Prioridade:**
- 🔴 **Alta**: 10 tasks (Tasks críticas para funcionalidade core)
- 🟡 **Média**: 9 tasks (Tasks importantes para completude)
- 🟢 **Baixa**: 1 task (Task de enhancement/usabilidade)

### **Por Task Group:**
- **Task 2.1** (Package Management): 5 tasks - 5 concluídas (✅ 100% COMPLETA)
- **Task 2.2** (Dependency Analysis): 5 tasks - 0 concluídas (0% completa)
- **Task 2.3** (Graph Construction): 5 tasks - 0 concluídas (0% completa)
- **Task 2.4** (Standard Integration): 5 tasks - 0 concluídas (0% completa)

### **Arquivos Principais para Criar/Modificar:**
- `crates/pkg/src/package/manager.rs` - Novo arquivo
- `crates/pkg/src/dependency/analyzer.rs` - Novo arquivo
- `crates/pkg/src/graph/builder.rs` - Refactor existente
- `crates/pkg/src/graph/validation.rs` - Enhance existente
- `crates/pkg/src/lib.rs` - Exports das novas APIs

### **Estruturas de Dados Principais:**
```rust
// Package Management
pub struct PackageManager<F: AsyncFileSystem>;
pub struct ValidationReport;

// Dependency Analysis  
pub struct DependencyAnalyzer;
pub struct DependencyReport;
pub struct Conflict;
pub enum DependencyClass;

// Graph Construction
// (usar estruturas existentes + enhancements)
```

---

## 🎯 **Próximo Passo**

🚀 **INÍCIO**: Task 2.1.1 - Criar estrutura base PackageManager

**Ordem de Execução Recomendada:**
1. **Task 2.1.x** (Package Management) - Base fundamental
2. **Task 2.4.1-2** (AsyncFileSystem + ProjectDetector) - Integração essencial  
3. **Task 2.2.x** (Dependency Analysis) - Análise sobre base sólida
4. **Task 2.3.x** (Graph Construction) - Construção com todos components
5. **Task 2.4.3-5** (Finalizar Integration) - Integração completa

### **Dependências entre Tasks:**
- Task 2.1.1-2 são pré-requisitos para Task 2.4.1
- Task 2.4.1-2 são pré-requisitos para Task 2.2.x
- Task 2.2.x é pré-requisito para Task 2.3.x
- Todas tasks são pré-requisitos para Task 2.4.5

---

**📅 Estimativa**: 1.5 semanas (Sprint 2 do roadmap)  
**🎯 Meta de Sucesso**: APIs core implementadas, integração completa com standard crate, cobertura de testes > 80%

**🔄 Criado**: 2025-07-28 - Fase 2 planejada com 20 tasks detalhadas  
**👤 Responsável**: AI Assistant  
**📋 Plano Base**: `/crates/pkg/Plan.md` - Fase 2