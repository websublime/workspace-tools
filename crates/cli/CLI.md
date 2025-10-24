# DRAFT PARA PRD (PRODUCT REQUIREMENTS DOCUMENT)

## INTENÇÃO

Vamos criar uma aplicação CLI para interagir com os crates deste projecto, nomeadamente a utilização do crate pkg (sublime_package_tools) e os restantes crates (standard e git) para complementar necessidades e decisões ou até info que sejam necessários dar ao crate principal (pkg).
O foco principal é que o cli facilite a execução de ações/tarefas na qual a pi do crate pkg providencia.

O utilizador/developer bem como git hooks ou actions do github vai chamar este cli para executar tarefas que sejam necessárias executar no nosso projecto. Para perceberes melhor o contexto vou dar vários exemplos de iteração com o cli.

## Exemplos de utilização

### A
O developer inicia um novo projecto (single ou monorepo) e após isto corre o cli para que este produza o ficheiro inicial de configuração (default) bem como leva com um prompt na qual lhe faz as seguintes questões:

- Qual a directoria de changesets? por defeito é .changesets/
- Que ambientes quer disponiveis? Development, stage, stage, test whatever. Aqui seria bom que ele fosse introduzindo o que pretendesse até dizer temos todos.
- Que ambeinte quer por defeito? Escolha feita da lista anteriormente introduzida, pode ser mais que um.
- Que tipo de estratégia quer para versionamento: independente ou unificada?
- Qual a url do registry? por defeito https://registry.npmjs.org
- Qual o tipo de formato que quer para o ficheiro de configuração? Escolhe um entre os seguintes: JSON, TOML ou YAML

Depois de ter os valores desta resposta gera o ficheiro de configuração na root do projecto com o nome repo.config.(extensão escolhida). Podes usar do crate pkg esta funcionalidade.

### B
O developer inicia o desenvolvimento de uma nova feature e cria uma nova branch, esta branch ao ser criada o hook de git chama o cli para o seguinte: o cli verifica se já tem o devido ficheiro de changeset criado ou não caso não tenha deverá perguntar ao ao dev o seguinte:

- Em que ambientes queres a tua feature? (lista dos ambientes criados no ficheiro de configuração)
- Qual o tipo de bump vai receber quando for merged para a main? (patch, minor, major)

Feito isto cria o ficheiro de changeset na directoria definida no ficheiro de configuração. Mais uma vez pkg fornece api.

### C
O developer vai trabalhando na feature branch a dada altura comita as alterações. O hook de git deve chamar o cli para competar o seguinte: o cli lê o ficheiro de changeset e verifica se o conteúdo está correcto, adiciona commit id á lista bem como o package que sofreu alteração. Isto é tudo automático sem necessidade de prompt do user usando as funcionalidades de changeset do crate pkg.

### D
O developer quer obter uma auditoria completa do projecto. Corre o comando audit e deve receber um prompt na qual lhe diz:
- Que formato de report quer? ver as possibilidades dada pelo crate package
- Que tipos de audit? por defeito todos ou uma lista do que o crate oferece
- Executar o audit e dar o tipo de report.

### E
O developer quer proceder com actualizações de packages no projecto. O cli faz uma busca e check obtendo informação do que pode ser upgraded. Se tiver dá um report ao dev e pergunta se quer proceder com as actualizações. Ver crate pkg para toda a funcionalidade existente acerca disto.

### F
O developer decide fazer merge da sua branch no github. Uma action disparada após o merge executa o cli para este proceder com o bump de versão dos packages alterados, criação de tag, criação ou actualização dos devidos changelogs. Os changesets activos são todos executados dando as devidas infos ás ações aqui referidas e posteriormente são arquivados.

Aqui temos alguns exemplos de utilização do nosso cli. As tools e ações são só para clarificar os exemplos, ou seja, o nosso cli é agnoóstico se corre em hook, actions de pipeline ou user.

### G
Uma das primeiras actions a correr na pipeline pode querer obter informações para partilhar para outras actions. Por exemplo. Corre o bump em modo dry run e obtem os packages que tem alterações, o tipo de bump (main branch: major, minor, patch - feature: dá snapshot versions). Podemos ter necessidade imagina de fazer deploy da feature branch, bem como usar a versão para fazer upload dos assets para um bucket etc. Ou seja esta action será uma das primeiras actions a correr para dar info em json. (Info Action)

## Especificações do cli

O CLI deverá ter dois para metros globais: root (opcional e por defeito a directoria onde está a ser executado) e tipo de log (info, debug, warn, error ou silenciado).

Depois cada subs comando poderá ter os seus especificos parametros e podem usar sempre os valores dos parametros globais se assim necessitarem.

Ou seja de um modo high level o cli deve fornecer um interface para executar as ffeatures presentes no crate pkg.

Alguns comandos de exemplo:

- wnt init (este pro exemplo pode ser o exemplo A e se tiver todos os parametros das perguntas necessárias o user não precisa de levar com os prompts)

- wnt changeset (mesma coisa se tiver os parametros não precisa de fazer prompt)
- wnt audit
- wnt upgrade (pode ter parametro dry-run)
- wnt bump (vários parametros opcionais bem como apenas só output de dry-run sem alterações, podemos querer uma snapshot version ou pre-release etc)

Aqui temos algumas das features que o cli deve providenciar. A ideia é que o cli seja um interface para o crate pkg e que este possa ser extendido com funcionalidades dos outros crates (standard e git) caso seja necessário. 

É finalidade é dissecar o que pudemos oferecer de comandos ao user que o crate pkg oferece como funcionalidade, estes são só alguns exemplos mas o teu trabaçho é perceber o crate pkg e dar mais opções de funcionalidades do cli.

Iteramos o documento até termos um PRD final e completo.

## Resumo final

Para este crate queremos criar um cli com estilo moderno e minimal, com um header do nome do programa e versão corrente. Certamente vamos ter um módulo de ui/components a usar. Usaremos os crates clap e command e vamos avaliar outros para componentes ou estilos. Por exemplo este projecto tem uma implementação bem clean e minimal: https://github.com/42futures/firm/blob/main/firm_cli/Cargo.toml. Mas commo vimos vamos ter necessidade certamente de alguns componentes mais complexos tipo: lista, table, input, progresso, escolha multipla etc, claro que não precisamos de uma panoplia dde componentes os estilos só mesmos os necessários. Pretendo que o user receba sempre no terminal o que está a acontecer como escolha/ação do comando. 

Avalia detalhadamente o crate pkg para iterarmos e definirmos o que o cli vai ter como features, Claro que os outros crates standard e git podem ser usados se virmos a necessidade bem como podes dar a tua opinião sobre features que o CLI pode ter.

Por fim o nosso CLI deverá ser suportado pelas 3 plataformas windows, linux e osx e quero também ter um script que via curl instale o binario.