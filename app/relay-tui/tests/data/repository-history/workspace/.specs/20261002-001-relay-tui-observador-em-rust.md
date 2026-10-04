# 20261002-001 - relay-tui: observador de terminal em Rust

## Problem

Quem trabalha com Relay mantem o harness em uma metade do terminal e precisa
ver, na outra, o estado duravel do workflow mudar sem sair do terminal. Falta um
painel estreito, apresentavel, que se instala com um unico download, sem
repositorio, runtime nem gerenciador de pacotes, e roda no macOS (Linux em
melhor esforco).

## Scope

- Novo binario `relay-tui` em `app/relay-tui/` (projeto Cargo), em Rust com
  `ratatui`, `crossterm` e `notify`. Argumentos parseados a mao:
  `--workspace <path>`, `--version`, `--help`. Sem opcao, observa o diretorio
  corrente.
- Passivo e read-only: `q`, `Esc` e `Ctrl-C` saem; nenhuma outra interacao.
  Nunca escreve em `.specs/` nem em `.orchestration/` e nunca lanca harness.
- Core em Rust (`core`): parse, as 13 verificacoes de integridade e a derivacao
  de `RelayState`, pura (conteudo entra, estado sai, sem disco), no contrato da
  ADR-0003. `workspace` le os registros e vigia `.orchestration/` e `.specs/`
  com debounce trailing de 150 ms.
- Fixtures em `app/relay-tui/tests/fixtures/<caso>/`: cada caso e um workspace
  em disco mais um `expected.json` com o `RelayState`. Um caso por status e um
  por verificacao de integridade.
- Visual em cartoes (cabecalho, Handoff, TODO, Backlog) com paleta One Dark e
  acento rosa Charm `#ff75bf` nos IDs; o fundo nao e pintado e vem do terminal.
  Verde = em andamento/concluido, azul = pronto/disponivel, amarelo =
  bloqueado, vermelho = inconsistente. A barra do TODO e segmentada por item
  (contagem, nunca porcentagem, ADR-0003). Estado inconsistente substitui o
  conteudo por um cartao de violacoes (`check` + `detail`). Sem rolagem: falta
  de altura trunca o TODO em "+N itens" e reduz o Backlog a uma linha de
  contagem; abaixo de cerca de 40 colunas mostra so cabecalho e status.
- Frescor visivel em texto: `atualizado`, `atualizando` (diretorios sujos, ultimo
  snapshot mantido) e redesenho no resize. O terminal e restaurado em saida
  normal, `Ctrl-C` e panic. Workspace sem `.orchestration/` mostra um cartao
  "nao e um workspace Relay" e continua vigiando.
- Distribuicao: GitHub Action disparada por tag publica um binario por alvo no
  Release. O requisito e o macOS (arm64 e x64), com `cargo test` no macOS no CI;
  Linux (x64 e arm64 com `musl`) e melhor esforco: e construido e testado
  quando possivel, e uma falha nele nunca bloqueia o release nem o CI. O Windows
  esta fora por enquanto. O contorno da quarentena do Gatekeeper fica
  documentado.
- ADR-0003, `app/relay-tui/DESIGN.md` com a paleta e uma linha em
  `app/AGENTS.md` e `README.md`.

## Non-goals

- Navegar, selecionar, lancar harness, editar ou escrever qualquer registro.
- Changelog e texto de spec na TUI.
- Detectar tema do terminal, terminal claro ou tema Omarchy.
- Assinatura e notarizacao de binario, auto-update, instaladores e pacotes
  (brew, nix).
- Exigir que Linux funcione para liberar algo, ou suportar o Windows nesta
  versao: o Linux e desejo, nao impedimento, e o Windows fica fora (ver a
  ADR-0003, decisao 6).
- Alterar a gramatica dos cinco registros.

## Decisions

**Rust com binario unico.** O requisito e um download sem runtime, leve e
multiplataforma. TypeScript compilado (`bun build --compile`) leva ~60-100 MB e
e menos testado com Ink; Go atenderia igualmente, e Rust foi escolhido por
preferencia do dono do projeto e porque o core e a peca mais facil de escrever
com seguranca, por ser pura e testada.

**Fixtures em formato neutro.** Cada caso e um workspace como qualquer
repositorio Relay e um `expected.json` escrito a mao, legiveis sem o crate; o
`core` precisa derivar exatamente o estado esperado, e o CI quebra numa
divergencia.

**Layout de cartoes e paleta One Dark + acento Charm.** Escolhidos por mockup
contra tipografico e faixa de status. Valores copiados, sem depender do crate
`ratatui-bubbletea-theme` (0.2, mantido por uma pessoa). Fundo herdado do
terminal para casar com o painel do harness ao lado.

**Inconsistente em vermelho.** O amarelo ja e de bloqueado; o estado que trava
o resto da tela ganha um tom so dele.

**Nome `relay-tui`, nao CLI.** E um observador read-only que nao interpreta
comandos; nao viola a regra de nao criar um Relay CLI antes de validar o
protocolo. A ADR-0003 registra o argumento.

**Passivo agora.** Navegacao fica para uma versao posterior, apos uso real.

**macOS primeiro.** O desenvolvimento e a validacao acontecem no macOS, que e a
maquina do dono do projeto; o Linux segue como diferencial desejado, mas melhor
esforco, e o Windows fica fora (como retomar: ADR-0003, decisao 6). O que nao e
testado fora do macOS nao e declarado funcionando: a documentacao diz qual alvo
e requisito e qual e melhor esforco.

## Acceptance criteria

- A-001 - ADR-0003 existe, registra a escolha de Rust, o core puro com o
  contrato do estado e os fixtures, o canal de distribuicao por binario e por
  que `relay-tui` nao e um Relay CLI; o `DESIGN.md` registra a paleta do
  terminal e o vermelho de inconsistente.
- A-002 - `app/relay-tui/tests/fixtures/` contem os sete casos de status e um
  caso por verificacao de integridade, cada um com workspace em disco e
  `expected.json`.
- A-003 - O `core` em Rust passa todos os fixtures e produz o `RelayState`
  esperado, incluindo `inconsistent` e contagens, sem acessar disco.
- A-004 - O `workspace` le os registros de um caminho explicito ou do diretorio
  corrente e uma rajada de escritas gera um unico snapshot apos 150 ms de
  quiescencia, coberto por teste.
- A-005 - A view desenha cada um dos sete estados e o cartao de violacoes
  conforme a paleta decidida, verificada por snapshots do `TestBackend` em 58 e
  40 colunas, sem comunicar estado apenas por cor.
- A-006 - O binario observa um workspace real: reflete uma mudanca de arquivo
  sem reiniciar, mostra `atualizando` e depois `atualizado`, redesenha no
  resize e restaura o terminal em `q`, `Ctrl-C` e panic.
- A-007 - O `relay-tui` nunca escreve em `.specs/` ou `.orchestration/` nem
  lanca processo, verificado por teste ou inspecao de codigo.
- A-008 - Uma tag publica no Release binarios para macOS arm64 e x64, e o CI
  roda `cargo test` no macOS: sao o requisito. Linux x64/arm64 e melhor esforco,
  e uma falha nele nao impede o release nem o CI; o Windows esta fora por
  enquanto. A documentacao traz o download, o contorno do Gatekeeper e diz quais
  alvos sao requisito.

## Backlog candidates

- B-001: ADR-0003 e paleta do terminal no `DESIGN.md`.
- B-002: Fixtures com workspace e estado esperado.
- B-003: Core em Rust passando os fixtures (cria o projeto Cargo).
- B-004: Workspace: leitura e watcher com debounce.
- B-005: View em cartoes para todos os estados.
- B-006: Binario completo: argumentos, laco de eventos e restauracao do terminal.
- B-007: Release multiplataforma, CI e documentacao de uso.
