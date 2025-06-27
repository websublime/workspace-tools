# Configuração Dinâmica de Repositório

A partir de agora, o sistema de changelog suporta configuração dinâmica para diferentes provedores de repositório Git, resolvendo o problema de URLs hardcoded apenas para GitHub.

## ✅ Problema Resolvido

**Antes:** O método `create_template_variables` tinha lógica hardcoded:
```rust
// ❌ Problemático - apenas GitHub
let web_url = if remote_url.starts_with("git@") {
    remote_url.replace("git@github.com:", "https://github.com/")
} else {
    // ...
};
```

**Agora:** Sistema configurável para múltiplos provedores:
```rust
// ✅ Configurável e extensível
let repo_config = &self.config_provider.config().git.repository;
let web_url = repo_config.detect_repository_url(remote_url);
```

## 🏗️ Arquitetura da Solução

### 1. **GitConfig Estendido**
```rust
pub struct GitConfig {
    // ... campos existentes
    pub repository: RepositoryHostConfig,  // 🆕 Nova configuração
}
```

### 2. **Configuração de Repositório**
```rust
pub struct RepositoryHostConfig {
    pub provider: RepositoryProvider,       // GitHub, GitLab, etc.
    pub base_url: String,                  // "github.com", "gitlab.company.com"
    pub url_patterns: UrlPatterns,         // Templates de URL
    pub auto_detect: bool,                 // Auto-detectar provider
    pub url_override: Option<String>,      // Override manual
}
```

### 3. **Provedores Suportados**
```rust
pub enum RepositoryProvider {
    GitHub,          // github.com
    GitHubEnterprise,// GitHub Enterprise Server  
    GitLab,          // gitlab.com + instâncias custom
    Bitbucket,       // bitbucket.org
    AzureDevOps,     // Azure DevOps / TFS
    Custom,          // Provedores customizados
}
```

## 📋 Configuração por Provedor

### GitHub (padrão)
```toml
[git.repository]
provider = "GitHub"
base_url = "github.com"
auto_detect = true

[[git.repository.url_patterns.ssh_conversions]]
ssh_pattern = "git@github.com:"
https_replacement = "https://github.com/"
```

### GitHub Enterprise
```toml
[git.repository]  
provider = "GitHubEnterprise"
base_url = "github.company.com"
auto_detect = true

[[git.repository.url_patterns.ssh_conversions]]
ssh_pattern = "git@github.company.com:"
https_replacement = "https://github.company.com/"
```

### GitLab
```toml
[git.repository]
provider = "GitLab"
base_url = "gitlab.com"
auto_detect = true

[git.repository.url_patterns]
commit_url = "https://{base_url}/{owner}/{repo}/-/commit/{hash}"
compare_url = "https://{base_url}/{owner}/{repo}/-/compare/{from}...{to}"

[[git.repository.url_patterns.ssh_conversions]]
ssh_pattern = "git@gitlab.com:"
https_replacement = "https://gitlab.com/"
```

### GitLab Custom/Enterprise
```toml
[git.repository]
provider = "GitLab"
base_url = "gitlab.company.com"
auto_detect = true

[[git.repository.url_patterns.ssh_conversions]]
ssh_pattern = "git@gitlab.company.com:"
https_replacement = "https://gitlab.company.com/"
```

### Bitbucket
```toml
[git.repository]
provider = "Bitbucket"
base_url = "bitbucket.org"

[git.repository.url_patterns]
commit_url = "https://{base_url}/{owner}/{repo}/commits/{hash}"
compare_url = "https://{base_url}/{owner}/{repo}/branches/compare/{to}..{from}"
```

### Azure DevOps
```toml
[git.repository]
provider = "AzureDevOps"  
base_url = "dev.azure.com/myorg"

[git.repository.url_patterns]
commit_url = "https://dev.azure.com/myorg/{owner}/_git/{repo}/commit/{hash}"
```

## 🔧 Uso Programático

### Criação de Configurações
```rust
use sublime_monorepo_tools::config::types::git::RepositoryHostConfig;

// GitHub Enterprise
let config = RepositoryHostConfig::github_enterprise("github.company.com");

// GitLab custom
let config = RepositoryHostConfig::gitlab_custom("gitlab.company.com");

// Bitbucket
let config = RepositoryHostConfig::bitbucket();

// Azure DevOps
let config = RepositoryHostConfig::azure_devops("myorg");
```

### Conversão de URLs
```rust
let config = RepositoryHostConfig::github_enterprise("github.company.com");

// SSH → HTTPS
let ssh_url = "git@github.company.com:team/project.git";
let https_url = config.detect_repository_url(ssh_url);
// Result: "https://github.company.com/team/project"

// Geração de URLs
let commit_url = config.generate_commit_url(&https_url, "abc123");
// Result: "https://github.company.com/team/project/commit/abc123"
```

## 🚀 Benefícios

### ✅ **Flexibilidade**
- Suporta todos os principais provedores Git
- Configuração específica por ambiente
- Extensível para provedores customizados

### ✅ **Compatibilidade**
- Mantém compatibilidade com código existente
- GitHub continua sendo o padrão
- Fallbacks inteligentes

### ✅ **Robustez**
- Sem hardcoding de URLs
- Seguimento das regras CLAUDE.md (sem unwrap, documentação completa)
- Logging detalhado para debugging

### ✅ **Escalabilidade**
- Facilmente extensível para novos provedores
- Configuração reutilizável em outros módulos
- Padrões consistentes

## 🧪 Exemplos de Teste

O sistema inclui testes abrangentes:

```bash
# Executar testes de configuração
rustc test_repository_config.rs -L target/release/deps \
  --extern sublime_monorepo_tools=target/release/libsublime_monorepo_tools.rlib
./test_repository_config
```

**Saída esperada:**
```
🧪 Testing Repository Configuration Functionality

📦 Testing GitHub Configuration
  SSH: git@github.com:owner/repo.git -> HTTPS: https://github.com/owner/repo
  Commit URL: https://github.com/owner/repo/commit/abc123def
  ✅ GitHub tests passed

🏢 Testing GitHub Enterprise Configuration  
  SSH: git@github.company.com:team/project.git -> HTTPS: https://github.company.com/team/project
  Commit URL: https://github.company.com/team/project/commit/def456ghi
  ✅ GitHub Enterprise tests passed

# ... demais provedores

✅ All repository configuration tests passed!
```

## 📝 Migração

Para projetos existentes, a migração é **automática**:

1. **Sem configuração:** Usa GitHub como padrão (comportamento atual)
2. **Com configuração:** Usa o provedor configurado
3. **Fallback:** Se conversão falha, tenta limpeza básica de URLs

## 🔍 Logging e Debug

O sistema inclui logging detalhado:

```rust
log::debug!("Found git remote URL: {}", remote_url);
log::debug!("Converted repository URL: {}", web_url);
log::warn!("Failed to convert repository URL '{}' using provider: {:?}", remote_url, provider);
```

---

**Resultado:** Sistema robusto, flexível e configurável que resolve definitivamente o problema de URLs hardcoded para GitHub, suportando enterprise e outros provedores Git! 🎉