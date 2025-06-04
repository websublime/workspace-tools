# Research Overview

## Introdução

Vamos supor que eu criei um CLI para agilizar determinadas tarefas e vou te dar alguns exemplos na qual podes tentar perceber se este crate consegue responder. Vamos imaginar que o nosso CLI permite:

- Criar um monorepo de um repositório template
- Após criação cria configs necessários
- O CLI permite ao dev criar workspace e packages novos a partir de templates
- Usar o config como espécie de registo para não ter de andar constantemente a correr todo o monorepo a procura de informações
- Config que dá para o nosso high level crate bem como as opções para os crate base
- Permitir ao dev configurar template de changelog, ele pode gostar de ter icones e fancy things
- Correr analises no monorepo e diz ao dev que pode actualizar packages e tem um problema X no package A
- Quando faço commit um hook determina as changes e cria o changeset perguntando ao dev em que ambiente quer colocar essa snapshot, guardando essa info. Quando for merged isto será apagado visto que a branch já não existe mais.
- Quando faço push o hook corre tarefas como testes etc nos apenas nos packages alterados
- O dev a determinado momento corre um comando do cli para previsualizar o graph de dependencias
- O dev a determinado momento quer saber quais dependencias vão ser alteradas
- O dev pretende pré visualizar o versionamento de internos, externos, dependencias e dependentes de.
- No Ci o merge aconteceu, após isso uma action vai fazer os bumps devidos, criar a tag e actualizar tudo que tem actualizar, para que o dev quando for a main receber tudo actualizado.
- O CLI tem um daemon que recebe constantemente informações de tudo que está a acontecer no monorepo
- O daemon permite conexão para enviar as informações para uma plataforma web, um mcp server e também para audit, logs etc.

Tendo tudo isto em mente achas que o crate futurista CLI/Daemon etc tem o que é necessário do nosso monorepo crate? Detalha todos os teus passos para eu perceber e contempla as necessidades. O crate monorepo é uma lib rust. No futuro depois de termos isto bem implementado falaremos sobre o CLI, só para perceberes o pq e necessidade de toda a solução.

## Feature Overview

O que vamos fazer? Estou a partilhar contigo um projecto que estou a criar chamado Workspace Tools que é para gestão e manutenção de monorepos em npm, pnpm, yarn e para outras ferramentas especializadas nesse tipo de trabalho como o turbo da vercel. Este monorepo tem os seguintes crates:

- `sublime-standard-tools`: O crate sublime_standard_tools fornece um conjunto abrangente de utilitários para trabalhar com projetos Node.js a partir de aplicações Rust. Ele gerencia a detecção de estrutura de projeto, execução de comandos, gerenciamento de ambiente e várias outras tarefas necessárias ao interagir com ecossistemas Node.js.
- `sublime-package-tools`: O crate sublime_package_tools fornece ferramentas abrangentes para gerenciar pacotes Node.js, dependências e tratamento de versões em aplicações Rust. Ela suporta análise de grafos de dependência, interações com registros de pacotes, comparações de versões e atualizações automatizadas de dependências.
- `sublime-git-tools`: O crate sublime_git_tools é uma interface Rust de alto nível para operações Git com tratamento robusto de erros, construída sobre o libgit2. Ele fornece uma API amigável para trabalhar com repositórios Git, encapsulando a poderosa mas complexa biblioteca libgit2 para oferecer uma interface mais ergonômica para operações Git comuns.

Posteriormente neste mesmo documento irei colocar a spec api de cada um dos crates para que possas entender melhor as suas funcionalidades e como utilizá-los.

O que pretendemos?

Pretendemos criar uma libraria/crate que permita gerir monorepos de forma eficiente, integrando com as ferramentas mais populares do ecossistema JavaScript/Node.js, como npm, pnpm, yarn e turbo. A ideia é fornecer uma interface unificada e fácil de usar para realizar tarefas comuns de manutenção e gestão de monorepos, como instalação de dependências, execução de scripts, atualização de pacotes, gestão e criação de versões dos pacotes internos, identificação de workspaces no monorepo etc.

A criação do novo crate `sublime-monorepo-tools` será o ponto central deste projecto, integrando as funcionalidades dos outros crates e fornecendo uma interface unificada para os utilizadores(developers). Este crate irá encapsular a lógica de gestão de monorepos, permitindo que os utilizadores possam usar na criação de clis, tuis, daemos e comunicação para aplicações web. Posteriormente iremos criar o daemon e o cli que irá consumir esta api.

## Regras de Código e estruturas

Os seguintes princípios devem ser seguidos ao escrever código para este projecto:

```rust
#![doc = include_str!("../SPEC.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
```
- **Documentação**: Todo o código deve ser bem documentado, seguindo as diretrizes de documentação do Rust. Utilize `///` para documentação de funções, structs e módulos.
- **Erros**: Utilize o tipo `Result` para tratar erros de forma adequada. Evite usar `unwrap` ou `expect` a menos que seja absolutamente necessário.
- **Clippy**: Utilize o Clippy para verificar o código e seguir as recomendações. Corrija os avisos do Clippy sempre que possível.
- **Testes**: Escreva testes para todas as funcionalidades importantes. Utilize o framework de testes do Rust para garantir que o código funciona como esperado.
- **Estilo de Código**: Siga as convenções de estilo do Rust. Utilize `rustfmt` para formatar o código automaticamente.
- **Segurança**: Evite práticas inseguras, como o uso de `unsafe` a menos que seja absolutamente necessário. Sempre que usar `unsafe`, documente o motivo e as garantias de segurança.
- **Performance**: Escreva código eficiente e robusto, mas não sacrifique a legibilidade por performance.

Exemplo:
```rust
//! # sublime_git_tools
//!
//! A high-level Rust interface to Git operations with robust error handling, built on libgit2.
//!
//! ## Overview
//!
//! `sublime_git_tools` provides a user-friendly API for working with Git repositories. It wraps the
//! powerful but complex libgit2 library to offer a more ergonomic interface for common Git operations.
//!
//! This crate is designed for Rust applications that need to:
//!
//! - Create, clone, or manipulate Git repositories
//! - Manage branches, commits, and tags
//! - Track file changes between commits or branches
//! - Push/pull with remote repositories
//! - Get detailed commit histories
//! - Detect changes in specific parts of a repository
//!
//! ## Main Features
//!
//! ### Repository Management
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new repository
//! let repo = Repo::create("/path/to/new/repo")?;
//!
//! // Open an existing repository
//! let repo = Repo::open("./my-project")?;
//!
//! // Clone a remote repository
//! let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Branch and Commit Operations
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let repo = Repo::create("/tmp/example")?;
//! // Create a new branch
//! repo.create_branch("feature/new-feature")?;
//!
//! // Checkout a branch
//! repo.checkout("feature/new-feature")?;
//!
//! // Add files and commit
//! repo.add("src/main.rs")?;
//! let commit_id = repo.commit("feat: update main.rs")?;
//!
//! // Or add all changes and commit in one step
//! let commit_id = repo.commit_changes("feat: implement new feature")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### File Change Detection
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let repo = Repo::create("/tmp/example")?;
//! // Get all changed files since a tag or commit
//! let changed_files = repo.get_all_files_changed_since_sha("HEAD~1")?;
//!
//! // Get all changed files with their status (Added, Modified, Deleted)
//! let changed_files_with_status = repo
//!     .get_all_files_changed_since_sha_with_status("HEAD~1")?;
//!
//! // Get changes in specific packages since a branch
//! let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
//! let package_changes = repo
//!     .get_all_files_changed_since_branch(&packages, "main")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Commit History
//!
//! ```rust
//! use sublime_git_tools::Repo;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let repo = Repo::create("/tmp/example")?;
//! // Get all commits since a specific tag
//! let commits = repo.get_commits_since(
//!     Some("HEAD~1".to_string()),
//!     &None
//! )?;
//!
//! // Get commits affecting a specific file
//! let file_commits = repo.get_commits_since(
//!     None,
//!     &Some("src/main.rs".to_string())
//! )?;
//! # Ok(())
//! # }
//! ```

#![doc = include_str!("../SPEC.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

mod repo;
mod types;

#[cfg(test)]
mod tests;

pub use types::{GitChangedFile, GitFileStatus, Repo, RepoCommit, RepoError, RepoTags};
```


Estruturas dos módules e crate.

A organização dos ficheiros deve seguir o seguinte exemplo:

```ascii
sublime-monorepo-tools
├── Cargo.toml
├── src
│   ├── lib.rs
│   ├── changes
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── changes.rs
│   │   ├── tests.rs
│   ├── config
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── tests.rs
│   ├── tasks
│   │   ├── mod.rs
│   │   ├── tasks.rs
│   │   ├── tests.rs
│   ├── versioning
│   │   ├── mod.rs
│   │   ├── ....rs
│   ├── error
│   │   ├── mod.rs
│   │   ├── changes.rs
│   │   ├── config.rs
│   │   ├── tasks.rs
│   │   ├── versioning.rs
...
```

Não precisa ser restritivo a esta estrutura mas sempre que possivel usar esta estrutura. Os struct devem ser definidos no ficheiro tpes.rs de cada módulo e sua implementações em ficheiro á aprte para melhor organização. Os testes devem ser colocados no ficheiro tests.rs de cada módulo, e devem ser escritos de forma a cobrir as funcionalidades principais do módulo.

É crucial entender as apis dos crates já existentes (`sublime-standard-tools`, `sublime-package-tools` e `sublime-git-tools`) para garantir que o novo crate `sublime-monorepo-tools` possa integrar-se corretamente com eles, deforma a evitar redundâncias e garantir uma experiência de desenvolvimento fluida.

## Dependencias

Neste rust monorepo existem as seguintes dependencias:

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
authors = ["WebSublime"]
edition = "2021"
rust-version = "1.82.0"
license = "MIT"
repository = "https://github.com/websublime/workspace-node-tools"

[workspace.dependencies]
# Internal crates
sublime_standard_tools = { version = "0.1.0", path = "./crates/standard" }
sublime_node_tools = { version = "0.1.0", path = "./crates/node" }
sublime_git_tools = { version = "0.1.0", path = "./crates/git" }
sublime_package_tools = { version = "0.1.0", path = "./crates/pkg" }
sublime_monorepo_tools = { version = "0.1.0", path = "./crates/monorepo" }

# Core dependencies
async-trait = "0.1.74"
chrono = { version = "0.4.31", features = ["serde"] }
derive_more = { version = "2.0.1", features = ["debug"] }
dirs = "6.0.0"
log = "0.4.20"
mimalloc = "0.1.42"
regex = "1.11.0"
serde = { version = "1.0.203", features = ["derive"] }
serde_json = "1.0.117"
tempfile = "3.8.1"
thiserror = "1.0.64"
tokio = { version = "1.38.0", default-features = false }
toml = "0.5.0"
walkdir = "2.4.0"

[workspace.lints.rust]
# Will be expanded as needed

[workspace.lints.clippy]
# Guidelines
dbg_macro = "deny"
print_stdout = "deny"
allow-dbg-in-tests = "allow"
clone_on_ref_ptr = "deny"
empty_drop = "deny"
exit = "deny"
filetype_is_file = "deny"
get_unwrap = "deny"
rc_buffer = "deny"
rc_mutex = "deny"
rest_pat_in_fully_bound_structs = "deny"
unnecessary_safety_comment = "deny"
pedantic = { level = "deny", priority = -1 }
module_inception = "allow"
module_name_repetitions = "allow"
similar_names = "allow"
missing_errors_doc = "allow"
must_use_candidate = "allow"
doc_markdown = "allow"
missing_const_for_fn = "allow"
needless_for_each = "allow"
new_without_default = "allow"
missing_panics_doc = "allow"
inconsistent_struct_constructor = "allow"
single_match = "allow"
single_match_else = "allow"

[profile.release]
codegen-units = 1
debug = false
lto = "fat"
opt-level = 3
strip = "symbols"

[profile.release-debug]
debug = true
inherits = "release"

[profile.release-wasi]
codegen-units = 16
debug = 'full'
inherits = "release"
lto = "thin"
opt-level = "z"
strip = "none"

```

## Pedido

Deliniar um plano de desenvolvimento para o crate `sublime-monorepo-tools`, um plano detalhado e erobusto para consumo de agentes que incluindo as seguintes etapas:

1. **Definição da API**: Especificar a API do crate, incluindo as funções principais, propriedades, tipos, traits e como elas irão interagir com os outros crates.
2. **Visualização do Fluxo de Trabalho**: Criar diagramas ou fluxogramas para visualizar o fluxo de trabalho do monorepo e como as diferentes funcionalidades irão interagir.
3. **Implementação Inicial**: Começar a implementação das funcionalidades básicas, como identificação da root do monorepo, leitura de ficheiros de configuração e integração com os crates existentes.
4. **Testes e Validação**: Escrever testes para garantir que as funcionalidades estão a funcionar como esperado e validar a integração com os outros crates.
5. **Erros**: Implementar um sistema de tratamento de erros robusto, utilizando o tipo `Result` e definindo erros específicos para as operações do monorepo e integrar os errors dos crates base (standard, package e git).
6. **Documentação**: Documentar a API e as funcionalidades do crate, incluindo exemplos de uso e guias de integração bem como um ficheiro SPEC que exponha toda a funcionilidade da API.

Posteriormente ao iteramos no chat eu partilho a api spec de cada um dos crates para que possas entender melhor as suas funcionalidades e como utilizá-los. A ideia é que possamos iterar juntos na construção deste crate, garantindo que ele atenda às necessidades dos utilizadores e seja fácil de usar.