# 20261002-004 - relay-tui: sugestao de proximo passo

## Problem

O painel diz onde o trabalho esta, mas nao o que fazer em seguida. Quem olha o
painel (principalmente depois de uma pausa) tem de lembrar qual skill chamar e
qual item vem agora. Essa resposta ja esta nos registros: o painel pode
mostra-la, sem executar nada.

## Scope

- Uma linha de **proximo passo** na visao Agora, acima do rodape, derivada so do
  estado ja carregado: o que sugerir e qual skill chamar. A TUI sugere, nunca
  executa e nunca escreve; quem age e a pessoa, no harness.
- A sugestao por estado:

  | Estado | Sugestao |
  | --- | --- |
  | `idle` | Nada em andamento. Para comecar: `relay-spec` (uma ideia) ou `relay-setup` (instalar o protocolo). |
  | `backlog` | Escolha um item. O primeiro disponivel e `B-NNN` (titulo); peca ao harness `relay-session`. |
  | `ready` | Ha uma subtarefa disponivel: `T-NNN` (titulo). Peca ao harness `relay-session`. |
  | `in_progress` | Retome pelo harness com `relay-session`: o handoff de `<harness>` (ha N min) segue em `T-NNN`. |
  | `blocked` | Resolva o bloqueio e retome com `relay-session`. Condicao de retomada: `<texto do handoff>`. |
  | `done` | Tudo concluido. Para uma nova ideia: `relay-spec`. |
  | `inconsistent` | Os registros se contradizem. `relay-status` mostra o diagnostico e `relay-continue` ajuda a recuperar. |

- "O primeiro disponivel" e a recomendacao padrao do protocolo (a primeira entrada
  disponivel em ordem textual, `docs/PROTOCOL.md`); a sugestao so a nomeia, sem
  dizer que e a de maior prioridade.
- A linha cabe na largura: texto longo e cortado com `…`; em largura menor que 40
  colunas a sugestao some junto com o resto, como os cartoes; sem altura para ela,
  some antes do TODO ser cortado.
- Core em Rust: uma funcao pura `suggest(estado) -> Sugestao`, fora do
  `RelayState`, para nao mexer no contrato da ADR-0003 nem na suite de conformidade.
- Secao no design system e uma linha em `docs/TUI.md` e `docs/TUI.pt-BR.md`.

## Non-goals

- Executar a sugestao, abrir o harness ou copiar o comando; para isso ha a skill
  `relay-tui` (spec 20261002-003) e o proprio harness.
- Priorizar, estimar ou ordenar alem da recomendacao padrao do protocolo.
- Texto da sugestao configuravel; traducao para o ingles (a tela e em portugues).

## Decisions

**A sugestao e funcao do estado, nao do tempo.** Nao depende de relogio nem de
historico: mesma entrada, mesma linha. O "ha N min" do handoff reaproveita o
`relative_time` ja existente.

**Fora do `RelayState`.** O contrato ADR-0003 e a suite de conformidade ficam como
estao: a sugestao e apresentacao de um estado que o core ja entregou pronto, nao
uma regra nova do protocolo. A escolha do "primeiro disponivel" usa o proprio
`available` vindo do core, sem recalcular dependencias.

**So uma linha.** Um cartao inteiro competiria com o handoff, que e o objeto
central da tela (principio 1 do design system); uma linha basta para dizer o
proximo passo.

## Acceptance criteria

- A-001 - Para cada um dos sete estados, a funcao `suggest` devolve a sugestao da
  tabela, nomeando o item, a skill e, em `blocked`, a condicao de retomada,
  verificado por testes sobre os casos de `app/conformance/`.
- A-002 - A linha de proximo passo se desenha na visao Agora em 58 e 40 colunas,
  cortada com `…` quando nao cabe e ausente abaixo de 40 colunas, verificado por
  snapshots.
- A-003 - A sugestao nunca e a unica forma de comunicar o estado e nao altera nada
  em disco, verificado pelos testes existentes de leitura-somente.
- A-004 - O design system descreve a linha e `docs/TUI.md` e `docs/TUI.pt-BR.md` a
  mencionam.

## Backlog candidates

- B-001: Funcao pura `suggest` no core, com a tabela de sugestoes testada.
- B-002: Linha de proximo passo na visao Agora, snapshots, design system e guias.
