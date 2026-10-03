# 20261002-004 - relay-tui: sugestao de proximo passo

## Problem

O painel diz onde o trabalho esta, mas nao o que fazer em seguida. Quem olha o
painel (principalmente depois de uma pausa) tem de lembrar qual skill chamar e
qual item vem agora. Essa resposta ja esta nos registros: o painel pode
mostra-la, sem executar nada.

## Scope

- Uma linha de **proximo passo** na visao Agora, logo acima do rodape, derivada so
  do estado ja carregado: o que sugerir e qual skill chamar. A TUI sugere, nunca
  executa e nunca escreve; quem age e a pessoa, no harness. E a "dica de proximo
  passo derivada do estado" que a ADR-0009 (decisao 7) deixou como possibilidade
  menor; nao acrescenta tecla nem interacao.
- A sugestao por caso, na ordem em que os casos sao testados (o primeiro que
  casar vale). `B-NNN`, `T-NNN` e titulos vem do estado; "titulo" e o texto da
  entrada sem as anotacoes `spec:`/`needs:`:

  | Caso | Condicao | Sugestao |
  | --- | --- | --- |
  | nao e workspace | diretorio sem `.orchestration/` | Instale o protocolo com `relay-setup`. |
  | `inconsistent` | violacoes | Os registros se contradizem: `relay-status` mostra o diagnostico e `relay-continue` pode propor o reparo. |
  | `in_progress` | handoff `in_progress` | Retome `T-NNN` de `B-NNN` com `relay-session`. |
  | `blocked` com handoff | handoff `blocked` | Resolva o bloqueio de `T-NNN` (ver Handoff) e retome com `relay-session`. |
  | `blocked` sem handoff | TODO sem item disponivel | Nenhuma subtarefa disponivel em `B-NNN`: `relay-continue` mostra o bloqueio e as opcoes. |
  | `ready` | TODO com item disponivel | Comece `T-NNN` (titulo) com `relay-session`. |
  | `done` com TODO | TODO todo `[x]`, sem handoff, item ativo ainda `[ ]` no backlog | Subtarefas de `B-NNN` concluidas: feche o item com `relay-session`. |
  | `backlog` com disponivel | ha entrada de backlog disponivel | Proximo item disponivel: `B-NNN` (titulo). Comece com `relay-session`. |
  | `backlog` sem disponivel | nenhuma entrada disponivel | Nenhum item disponivel: resolva os bloqueios ou dependencias do backlog; `relay-continue` mostra as opcoes. |
  | `done` | qualquer outro `done` | Tudo concluido. Para uma nova ideia: `relay-spec`. |
  | `idle` | sem backlog, TODO nem handoff | Nada em andamento. Para comecar: `relay-spec`. |

- "Proximo item disponivel" e a recomendacao padrao do protocolo (a primeira
  entrada disponivel em ordem textual, `docs/PROTOCOL.md`); a sugestao so a
  nomeia, sem dizer que e a de maior prioridade. As recomendacoes acompanham as
  do `skills/relay-continue/SKILL.md` para o mesmo estado.
- A linha cabe na largura: texto longo termina em `…` em 58 e 40 colunas; abaixo
  de 40 colunas ela nao aparece (vale a linha de status da spec 20261002-001).
  Na falta de altura, ela cede depois de a area do backlog (o cartao Backlog de
  hoje ou os cartoes de spec da spec 20261002-002) virar uma linha e antes de o
  Handoff se compactar; logo, sempre antes de o TODO cortar.
- A skill vai em texto, em negrito `fg`; os ids em `id`; o resto da linha em
  `meta`. Nada depende so da cor.
- Funcao pura `suggest` num modulo proprio do crate (`src/suggest.rs`), **fora**
  de `core` e de `view`: recebe o `RelayState` ou a ausencia de workspace e
  devolve um valor estruturado (caso, skill, ids e titulo); o texto em portugues
  e montado pela view. Nao usa relogio nem disco.
- Secao no design system e uma linha em `docs/TUI.md` e `docs/TUI.pt-BR.md`.

## Non-goals

- Executar a sugestao, abrir o harness, copiar o comando ou acrescentar teclas;
  a TUI so mostra.
- Repetir na linha o que os cartoes ja mostram (harness, tempo relativo, texto do
  bloqueio).
- Priorizar, estimar ou ordenar alem da recomendacao padrao do protocolo.
- Texto da sugestao configuravel; traducao para o ingles (a tela e em portugues).
- Linha de sugestao na visao Histórico (spec 20261002-002).

## Decisions

**A sugestao e funcao do estado, nao do tempo.** Nao depende de relogio nem de
historico: mesma entrada, mesma linha. Por isso o "ha N min" do handoff fica so no
cartao Handoff, que ja o mostra com o relogio injetado.

**Fora do `RelayState` e fora do `core`.** O contrato ADR-0003 e a suite de
conformidade ficam como estao, e o `core` continua sendo o porte do `relay-core`
em TS que a suite mantem alinhado; uma funcao so do Rust ali faria o core dizer
mais que a referencia. A sugestao e apresentacao de um estado que o core ja
entregou pronto. A escolha do "proximo disponivel" usa o proprio `available` vindo
do core, sem recalcular dependencias.

**Casos alem dos sete status.** O status sozinho nao basta: o core deriva
`blocked` sem handoff (TODO sem item disponivel), `done` com o TODO todo feito e o
item de backlog ainda aberto (falta o passo 5 do protocolo) e `backlog` sem
nenhuma entrada disponivel. Uma sugestao por status mandaria "nova ideia" com um
item por fechar, ou nomearia um item que nao existe.

**`relay-setup` so fora de um workspace.** Em `idle` o protocolo ja esta
instalado; a skill de instalacao so faz sentido no cartao "nao e um workspace
Relay".

**So uma linha.** Um cartao inteiro competiria com o handoff, que e o objeto
central da tela (principio 1 do design system); uma linha basta para dizer o
proximo passo.

**Convivencia com a spec 20261002-002.** Aquela e dona do corpo de Agora (cartoes
de spec) e do rodape; esta so acrescenta a linha acima do rodape. Nao ha
dependencia: a que entrar depois regenera os snapshots de Agora.

## Acceptance criteria

- A-001 - `suggest` devolve, para cada caso da tabela, a skill e os ids e o
  titulo da tabela, verificado por testes sobre os sete casos de status de
  `app/conformance/` e por casos sinteticos para nao-workspace, `blocked` sem
  handoff, `done` com TODO e `backlog` sem entrada disponivel; e testes mostram
  que a mesma entrada da a mesma sugestao, sem relogio.
- A-002 - A linha de proximo passo se desenha na visao Agora em 58 e 40 colunas,
  cortada com `…` quando nao cabe, ausente em 30 colunas e, em snapshots de
  altura reduzida, ausente antes do Handoff compactado e do TODO cortado,
  verificado por snapshots.
- A-003 - A skill da sugestao aparece em texto e em negrito, nunca so em cor,
  verificado por teste de semantica da view; e o modulo `suggest` entra na lista
  do teste `read_only.rs` de modulos que nao leem disco.
- A-004 - O design system (secao 10) descreve a linha, a tabela de casos e a
  regra de altura, e `docs/TUI.md` e `docs/TUI.pt-BR.md` a mencionam.

## Backlog candidates

- B-001: Funcao pura `suggest`, fora do core, com a tabela de sugestoes testada.
- B-002: Linha de proximo passo na visao Agora, snapshots, design system e guias.
