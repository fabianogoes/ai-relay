# 20261003-001 - Changelog por spec, ciclo de vida da spec e contrato nas skills

## Problem

Quatro defeitos do protocolo, achados na análise de 2026-10-03, mais um que o
uso já cobrou.

**O changelog cresce sem limite.** `.orchestration/CHANGELOG.md` é um arquivo
único e append-only. Neste repositório ele chegou a ~2.400 linhas antes da
limpeza de 2026-10-03, e é lido para verificar critérios de aceite: o custo em
tokens que o README admite só aumenta. O arquivo único também concentra o
conflito de merge entre branches. O `BACKLOG.md` tem o mesmo problema em escala
menor: guarda para sempre uma linha por item concluído.

**O protocolo não sabe abandonar.** Não há como descartar um item ou uma spec.
Remover a interface web exigiu apagar 12 specs e 111 registros de um changelog
que deveria ser append-only. Pelo `AGENTS.md`, essa fricção é defeito do
`docs/PROTOCOL.md`.

**As skills instaladas não levam o contrato.** Só `./skills/` é distribuído, e
nenhuma skill contém os templates de handoff, TODO, changelog ou spec, nem cita o
`docs/PROTOCOL.md`. A transição 2 (criar o TODO a partir de uma entrada de
backlog) não aparece em skill nenhuma, e o `relay-status` nomeia 8 das 13
verificações. Usar o Relay neste repositório esconde o problema, porque aqui o
`AGENTS.md` aponta o protocolo. Fora daqui, um agente que escreva
`## Critérios de aceite` faz a verificação de critérios passar em silêncio, e um
cabeçalho de changelog fora do formato gera um `inconsistent` falso.

**O leitor erra em silêncio.** No `relay-tui`, uma linha `- Status:` dentro de
`## Context` sobrescreve os metadados e conta como segundo handoff. Um handoff
que contenha "No active handoff" em qualquer lugar é tratado como vazio. Um
status com erro de digitação vira `in_progress`. Um `BACKLOG.md` com CRLF lê
vazio e deriva `idle`. Essas regras vieram da paridade com um leitor TypeScript
que não existe mais.

**O protocolo tem lacunas.** Não verifica IDs repetidos, não diz nada sobre
branches, não define como alocar o próximo `B-NNN` e não nomeia o estado "TODO
todo `[x]`, sem handoff, entrada de backlog ainda aberta" (o core o chama de
`done`, que no protocolo é "backlog todo `[x]`").

## Scope

- **Changelog por spec.** `.orchestration/changelog/<YYYYMMDD-NNN>.md`, um
  arquivo por spec. Uma sessão lê só o arquivo da spec ativa.
- **Fechar é arquivar.** Ao fechar uma spec, as entradas dela saem do
  `BACKLOG.md` para uma seção de fechamento no changelog da spec. O
  `BACKLOG.md` guarda só trabalho aberto.
- **Descarte.** Marcador `[-]` com motivo para entradas de backlog; uma spec
  inteira pode ser descartada; critério sem evidência de spec com descarte é
  dispensado com motivo no fechamento.
- **Legado e migração.** Um `CHANGELOG.md` único continua legível; o
  `relay-setup` o migra sob confirmação.
- **Leitura estrita.** Metadados do handoff, handoff vazio, `Status` válido e
  CRLF definidos no protocolo e no core.
- **Lacunas.** IDs únicos, alocação de `B-NNN`, branches e o estado
  intermediário escritos no protocolo.
- **Contrato nas skills.** Cada skill que escreve ou valida carrega em
  `references/` os blocos do protocolo de que precisa, com teste de paridade.
- **relay-tui** lendo a estrutura nova: core, watcher, fixtures, Agora,
  Histórico, `DESIGN.md` e `docs/TUI.md`.
- **Este repositório migrado** para a estrutura nova.
- `docs/PROTOCOL.pt-BR.md` e `docs/TUI.pt-BR.md` acompanham o inglês no mesmo
  commit, como o `AGENTS.md` exige.

## Non-goals

- Os itens 5 a 10 da análise: horário obtido do shell, documentação duplicada
  entre `docs/TUI.md` e o README do crate, restos da interface web, CI, idioma
  da tela e `SIGTERM`. Ficam para outra spec.
- **Nenhum status novo.** A lista fixa (`backlog`, `ready`, `in_progress`,
  `blocked`, `done`, `idle`) não muda.
- **Nenhum Relay CLI**, e o `relay-tui` continua só lendo. A migração é feita
  por skill.
- **Nenhuma mudança no CI.** O teste de paridade roda em `.agents/tests/`;
  levá-lo ao CI exige mexer na conformidade 5 da ADR-0003 e é o item 8.
- **`.specs/` não muda de lugar.** Spec fechada continua no mesmo caminho, e todo
  caminho já citado em registro continua válido.
- **Descarte só no backlog.** Subtarefa de TODO não ganha `[-]`.
- Nenhum ID existente é renumerado. A política de ADR deste repositório não é
  parte do protocolo e não muda aqui.

## Decisions

**Um arquivo de changelog por spec** (escolha do usuário). Uma sessão lê só o
arquivo da spec ativa, cada arquivo cresce no máximo até o tamanho da sua spec,
specs em branches diferentes não conflitam, e a estrutura espelha o Histórico
(spec → itens → tarefas). Alternativas rejeitadas: um arquivo por registro (zero
conflito, mas centenas de arquivos pequenos) e rotação por período (não agrupa
por spec, a verificação de critério atravessa arquivos e o arquivo corrente
continua sendo ponto de conflito).

**O arquivo é a spec; o registro não repete `Spec`.** A coerência vem da
entrada de backlog: a spec da entrada nomeada em `Backlog` tem de ser a do
arquivo, e uma verificação nova acusa a divergência. Critério não qualificado
pertence à spec do arquivo; `YYYYMMDD-NNN/A-NNN` continua qualificando critério
de outra spec. Repetir o campo seria um segundo lugar para discordar.

**Fechar uma spec é arquivá-la** (escolha do usuário). Quando a última entrada
pendente fecha, o escritor acrescenta ao changelog da spec uma seção
`## Closed <data>` com as entradas finais copiadas literalmente (`[x]` ou
`[-]`) e as dispensas, e só então as remove do `BACKLOG.md`. **A seção de
fechamento é a autoridade:** uma entrada de backlog cujo ID já aparece no
fechamento da própria spec está arquivada, e removê-la do `BACKLOG.md` é
limpeza. Assim a janela entre as duas escritas não produz estado falso, e uma
sessão interrompida no meio deixa um estado coerente. Consequência: com tudo
fechado, o estado derivado é `idle`; o `done` por "backlog todo `[x]`" continua
valendo para backlogs anteriores a esta mudança. Alternativas rejeitadas: manter
as entradas `[x]` no `BACKLOG.md` (cresce para sempre) e arquivar por pedido
explícito (passo a lembrar, e o arquivo volta a crescer).

**Descarte com `[-]`, escrito pelo `relay-spec`.** Uma entrada descartada leva
`(dropped: <motivo>)` e nunca é apagada. Descartar muda o escopo, e escopo é
da spec. Uma entrada descartada não fica disponível e não satisfaz `needs`:
uma entrada pendente que precise de uma descartada é violação, de modo que
descartar exige ajustar ou descartar as dependentes. Descartar a última entrada
pendente fecha a spec. Alternativa rejeitada: apagar a spec e os registros, que
foi o que este repositório precisou fazer em 2026-10-03.

**Dispensa só com descarte.** `- Waived: A-NNN - <motivo>` existe apenas no
fechamento de spec com ao menos uma entrada `[-]`, e só o `relay-spec` a
escreve, por decisão do usuário. Sem essa restrição, dispensar viraria um atalho
para não provar um critério. Um critério que deixou de fazer sentido sem
descarte é mudança da especificação e sai da spec.

**Legado legível, migração sob confirmação.** Um `.orchestration/CHANGELOG.md`
existente continua válido para leitura e nunca recebe registro novo. O
`relay-setup` o divide por `Spec` em arquivos por spec, sob confirmação, movendo
cada registro sem alterar o texto; registros sem `Spec` ou com spec inexistente
ficam no arquivo legado e são reportados. Esta é a única movimentação de
registro que o protocolo permite, e ela preserva o texto. Segue o precedente de
nomes de spec antigos e de critérios com marcador, que continuam válidos.

**Alocação de `B-NNN`.** O próximo é um a mais que o maior `B-NNN` em qualquer
arquivo de `.orchestration/`. Inclui os arquivados e o legado, e o agente
resolve com um `grep`, sem ler os arquivos inteiros.

**IDs únicos.** Um `B-NNN` aparece uma vez entre o `BACKLOG.md` e as seções de
fechamento (fora a janela de fechamento da decisão acima), e um `T-NNN` aparece
uma vez no TODO. Uma verificação nova acusa a repetição.

**Leitura estrita.** Os metadados do handoff são as linhas `- Chave: valor`
antes do primeiro `##`; o que vem nas seções é texto livre. O handoff vazio é só
a forma vazia do template. `Status` aceita apenas `in_progress` e `blocked`, e
outro valor é violação. Leitores tratam `\r\n` como `\n`. Alternativa rejeitada:
manter as peculiaridades herdadas da paridade com o leitor TypeScript removido.
Hoje há um leitor só, e elas produzem estado errado em silêncio.

**Branches.** Os registros viajam com o branch e cada branch deriva o próprio
estado. Um branch é integrado com o handoff vazio. Uma colisão de `B-NNN` no
merge é renumerada no branch que entra, e a verificação de IDs repetidos a
denuncia. Alternativa rejeitada: registros fora do git, que desfaria a premissa
de o estado viver no repositório.

**O estado intermediário é `done` da tarefa ativa.** TODO todo `[x]`, sem
handoff e com a entrada de backlog aberta deriva `done`, e o próximo passo é a
transição 5. O core e a sugestão já fazem isso; o protocolo passa a dizer.
Alternativa rejeitada: um status `closing`, que ampliaria a lista fixa.

**Contrato embarcado em `references/`.** Cada skill que escreve ou valida
registros carrega, em `skills/<skill>/references/`, cópias literais dos blocos
do `docs/PROTOCOL.md` de que precisa (templates e verificações), e o `SKILL.md`
manda lê-las antes de escrever. Um teste em `.agents/tests/` falha quando uma
cópia diverge. Alternativas rejeitadas: symlink para o `docs/PROTOCOL.md` (sai de
`./skills/`, quebra a fronteira da ADR-0001 e não sobrevive a instaladores que
copiam só a skill) e o `relay-setup` copiar o protocolo para o repositório do
usuário (um arquivo a mais, desatualizado a cada versão das skills).

**Contrato antes de skills e core**, como o `AGENTS.md` exige: o
`docs/PROTOCOL.md` muda primeiro, e cada verificação nova entra na tabela da
ADR-0003 na mesma mudança. A estrutura do changelog ganha uma ADR própria, porque
muda uma interface em disco.

## Acceptance criteria

- A-001 - `docs/PROTOCOL.md` define o changelog como um arquivo por spec,
  `.orchestration/changelog/<YYYYMMDD-NNN>.md`, cujo registro não repete a spec,
  e diz que uma sessão lê só o arquivo da spec ativa
- A-002 - Fechar uma spec acrescenta a seção de fechamento ao changelog dela e
  tira as entradas do `BACKLOG.md`; uma entrada que já está no fechamento da
  própria spec é tratada como arquivada
- A-003 - Uma entrada de backlog pode ser descartada com `[-]` e motivo; fechar
  uma spec com descarte exige que todo critério sem evidência tenha uma dispensa
  com motivo
- A-004 - Um `CHANGELOG.md` único continua legível e não recebe registro novo; o
  `relay-setup` o migra sob confirmação, sem alterar o texto de nenhum registro,
  e uma segunda execução não muda nada
- A-005 - O protocolo define o próximo `B-NNN` como um a mais que o maior
  `B-NNN` em `.orchestration/`
- A-006 - Uma ADR nova registra a estrutura do changelog e o fechamento como
  arquivamento, com as alternativas rejeitadas
- A-007 - O protocolo define os metadados do handoff como as linhas antes do
  primeiro `##`, o handoff vazio como a forma vazia do template e `Status` como
  `in_progress` ou `blocked`; uma linha `- Status:` em `## Context` não altera o
  estado derivado
- A-008 - Um registro com CRLF deriva o mesmo estado que o mesmo registro com LF,
  e o limite "CRLF ignorado" sai do `docs/TUI.md`, do README do crate e da
  ADR-0003
- A-009 - Cada verificação nova (ID repetido, `Status` de handoff inválido,
  registro em changelog de outra spec, `needs` que aponta entrada descartada)
  tem identificador estável na tabela da ADR-0003 e um fixture
- A-010 - O protocolo tem uma seção sobre branches: registros viajam com o
  branch, o branch é integrado com o handoff vazio e uma colisão de `B-NNN` é
  renumerada no branch que entra
- A-011 - O protocolo nomeia o estado "TODO todo `[x]`, sem handoff, entrada
  aberta" como `done` da tarefa ativa com a transição 5 como próximo passo, e
  core, sugestão e skills seguem esse nome
- A-012 - O `relay-tui` deriva o estado de `changelog/*.md` e do legado, vigia
  `changelog/` e passa nos fixtures novos (por spec, legado, spec fechada,
  descarte, dispensa, CRLF) e nos antigos
- A-013 - O Histórico lista specs fechadas com as entradas arquivadas e mostra
  itens descartados com o motivo; a visão Agora não mostra descartados; o
  `DESIGN.md` define a aparência antes do código, com snapshots em 58 e 40
  colunas
- A-014 - Cada skill que escreve ou valida registros carrega em `references/` os
  blocos do protocolo de que precisa, e um teste em `.agents/tests/` falha quando
  um bloco diverge do `docs/PROTOCOL.md`
- A-015 - `relay-session` descreve a criação do TODO (transição 2) e o
  fechamento da spec; `relay-status` e `relay-continue` cobrem todas as
  verificações de integridade do protocolo
- A-016 - `relay-spec` descarta uma entrada ou uma spec inteira, com motivo, sem
  apagar registro
- A-017 - Nenhum `SKILL.md` passa de 39 linhas
- A-018 - Num repositório limpo, com só o plugin instalado e sem
  `docs/PROTOCOL.md`, um ciclo completo no Claude Code (setup, spec, sessão,
  fechamento da spec) produz registros que o `relay-tui` deriva sem violação
- A-019 - Este repositório usa a estrutura nova: changelog dividido por spec com
  os registros preservados, specs concluídas fechadas e estado derivado sem
  violação

## Backlog candidates

- B-051: Protocolo e ADR do changelog por spec, fechamento, descarte e migração
- B-052: Protocolo com leitura estrita, IDs únicos, branches e estado
  intermediário
- B-053: Core do relay-tui lê a estrutura nova e o legado, com as verificações
  novas e os fixtures (needs: B-051, B-052)
- B-054: Visões do relay-tui com specs fechadas e itens descartados
  (needs: B-053)
- B-055: Skills com o contrato em `references/` e teste de paridade
  (needs: B-051, B-052)
- B-056: `relay-spec` descarta; `relay-setup` migra o changelog legado
  (needs: B-055)
- B-057: Este repositório migrado e as specs concluídas fechadas
  (needs: B-053, B-056)
- B-058: Validação num repositório limpo só com o plugin (needs: B-054, B-056)
