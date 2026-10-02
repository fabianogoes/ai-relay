# 20261002-002 - relay-tui: navegacao pelo historico do trabalho

## Problem

O painel mostra so o agora. Para saber o que ja foi feito (quais specs, quais
itens de backlog, quais tarefas, e o que cada uma entregou) a pessoa abre os
arquivos ou a UI web. Isso deveria caber no proprio painel, em listas curtas que
se aprofundam por selecao: spec, backlog, tarefa, detalhe.

## Scope

- Uma segunda visao, **Trabalho**, ao lado de **Agora**, que continua sendo a
  padrao. `Tab` (ou `t`) alterna entre as duas, como na UI web (ADR-0007).
- Quatro niveis de lista, cada um aprofundando o anterior: **specs**, os
  **itens de backlog** da spec, as **tarefas** do item e o **detalhe** da tarefa.
  Selecionar uma linha abre o nivel seguinte; `Esc` ou `Backspace` volta; `Enter`
  e o **clique do mouse** abrem; as setas (e `j`/`k`) movem a selecao.
- Cada linha e um titulo conciso, de uma linha. Titulo que nao cabe e cortado com
  `…`; o detalhe mostra o titulo inteiro e o resto, quebrado pela largura.
- Spec: id, titulo (o `# AAAAMMDD-NNN - Titulo` do arquivo) e a contagem
  `feitos/total` dos seus itens de backlog, nunca porcentagem. Item de backlog:
  id, texto e o estado. Tarefa: id (`T-NNN`), titulo e data do registro do
  changelog. Detalhe da tarefa: o registro inteiro (Result, Evidence, Criteria,
  Decisions).
- Uma tarefa em andamento (do TODO atual) aparece na lista do seu item com o
  marcador de agora e sem registro ainda.
- A captura do mouse fica ligada so enquanto a visao Trabalho esta aberta, para a
  visao Agora continuar permitindo selecionar e copiar texto normalmente.
- Core em Rust: extracao estruturada do changelog (todos os campos do registro) e
  do titulo de cada spec. So extracao de texto: nenhuma regra nova do protocolo.
- ADR-0010 (emenda ao "passivo" da ADR-0009) e uma secao no design system.
- Continua somente leitura: a navegacao e estado local da tela.

## Non-goals

- Escrever, editar ou abrir arquivo no editor; busca e filtro; mostrar o texto
  integral da spec (a ADR-0007 decisao 3 ja o retirou da UI web).
- Paridade do parse novo no `relay-core` em TypeScript e caso de conformidade
  para ele (ver Decisions).
- Tema claro; Windows.

## Decisions

**A tarefa concluida vem do changelog.** O `TODO.md` e esvaziado quando o item de
backlog fecha, entao o unico registro de uma subtarefa feita e o do changelog
(`T-NNN`, titulo, Backlog, Spec, Result, Evidence). E o mesmo caminho da cascata
spec, backlog e changelog da UI web (ADR-0007 decisao 6).

**Mouse e teclado, os dois.** O pedido e clicar, e a captura do mouse tira a
selecao de texto do terminal; por isso ela so existe na visao Trabalho, e todo
gesto do mouse tem equivalente de teclado (acessibilidade e terminais sem mouse).

**O parse novo fica so no Rust.** E extracao de texto sem regra, entao o risco de
divergencia e baixo; fica coberto por testes contra os registros reais deste
repositorio, que tem dezenas de specs, backlogs e registros. Estender a suite de
conformidade e o `relay-core` e uma decisao a tomar se isso virar regra.

**Cabecalho e rodape guiam.** Em Trabalho o rodape lista as teclas do nivel
(`Enter abre · Esc volta · Tab Agora · q sai`), porque a visao deixa de ser so de
olhar.

## Acceptance criteria

- A-001 - A ADR-0010 registra a navegacao como estado local da tela, ainda somente
  de leitura, e o design system descreve a visao Trabalho: niveis, linhas, detalhe,
  teclas e cliques.
- A-002 - O core em Rust extrai dos registros reais deste repositorio a lista de
  specs com titulo, o backlog por spec e as tarefas por item com todos os campos
  do registro, verificado por testes.
- A-003 - `Tab` alterna Agora e Trabalho; em Trabalho, `Enter` ou clique entram no
  nivel seguinte, `Esc` ou `Backspace` voltam e as setas movem a selecao, verificado
  por testes do estado, sem terminal.
- A-004 - Cada nivel e o detalhe se desenham em 58 e 40 colunas, com titulos de uma
  linha cortados com `…` e o detalhe mostrando o texto inteiro, verificado por
  snapshots.
- A-005 - No binario real, num pty, um clique seleciona e abre o nivel seguinte, o
  terminal e restaurado ao sair e o workspace continua inalterado.
- A-006 - O README do relay-tui e o guia `docs/TUI.md` descrevem a visao Trabalho e
  suas teclas.

## Backlog candidates

- B-001: ADR-0010 e a visao Trabalho registradas no design system.
- B-002: Core em Rust extrai specs, backlog por spec e tarefas por item dos registros.
- B-003: Estado da navegacao (niveis, selecao, teclas e cliques) sem terminal.
- B-004: Listas e detalhe desenhados em 58 e 40 colunas, com snapshots.
- B-005: Binario com captura de mouse na visao Trabalho, teste em pty e documentacao.
