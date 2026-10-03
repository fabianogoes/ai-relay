# ADR-0004 — relay-tui: navegação como estado local da tela

## Status

**Accepted** — 2026-10-03.

Detalha a decisão 1 da ADR-0003: as teclas só mudam o que a tela mostra, e o
`relay-tui` nunca escreve um registro.

## Contexto

A visão Agora mostra o estado de agora: handoff, TODO e backlog. Para saber o
que já foi feito (quais specs, quais itens de backlog, quais tarefas e o que
cada uma entregou) seria preciso abrir os arquivos, embora o dado esteja todo
nos registros que o painel já lê.

Navegar por esse histórico no próprio painel, com teclado e com mouse, traz um
risco: a captura do mouse pelo terminal tira a seleção e a cópia de texto.

## Decisão

### 1. A navegação é estado local da tela, e continua somente leitura

Setas, `j`/`k`, `Enter`, `Esc`, `Backspace`, `Tab`/`t`, `r`, a roda e o clique
mudam só o que a tela mostra: a visão, o nível e a linha selecionada. Nenhuma
tecla ou clique escreve em `.specs/` ou em `.orchestration/`, abre editor ou
lança processo. `r` força uma releitura do workspace e nada mais. O módulo de
navegação não lê disco e entra na lista de módulos sem leitura do teste
`read_only.rs`.

### 2. Uma segunda visão, Histórico, ao lado de Agora

**Agora** continua sendo a visão padrão ao abrir. **Histórico** aprofunda por
seleção em quatro níveis (specs, itens de backlog da spec, tarefas do item e
detalhe da tarefa). A tarefa concluída vem do changelog, porque o `TODO.md` é
esvaziado quando o item de backlog fecha. A ordem, as linhas, a rolagem, a
largura mínima e as teclas de cada nível estão no `app/relay-tui/DESIGN.md`, que
tem autoridade sobre a aparência; esta ADR não as repete.

### 3. O nome é Histórico, e não Trabalho

"Trabalho" sugere o que está em curso, que é o que Agora já mostra; esta visão
é o passado e o que falta.

### 4. `Esc` volta em Histórico e sai em Agora

Em Histórico, `Esc` é o gesto natural de voltar; mantê-lo como saída faria perder
a navegação num toque. No nível de specs ele volta a Agora, e sair exige um
segundo `Esc` ou `q`. `q` e `Ctrl-C` saem de qualquer visão, na hora.

*Emenda de 2026-10-03, depois do uso real:* em Agora, `Esc` não sai mais sozinho,
porque um toque perdido encerrava o painel. Ele pergunta `Sair?` no rodapé; `Esc`,
`Enter` ou `y` confirmam e qualquer outra tecla cancela (e é consumida: não faz o
que faria). `q` e `Ctrl-C`, por serem gestos explícitos, continuam saindo na hora.

### 5. O mouse existe só em Histórico, e todo gesto tem equivalente de teclado

A captura do mouse é ligada ao abrir Histórico e desligada ao voltar a Agora, ao
sair e no panic, junto com a restauração do terminal. Assim Agora continua
permitindo selecionar e copiar texto. Todo gesto do mouse (clique abre o nível
seguinte, roda move a seleção) tem tecla equivalente, por acessibilidade e por
terminais sem mouse. Voltar um nível pelo mouse e botões clicáveis ficam fora.

### 6. A extração do histórico não é regra do protocolo

A navegação exige extrair do changelog data, título e todos os campos (com as
linhas de continuação) e o título de cada spec. É extração de texto, sem regra
nova do protocolo: `RelayState` e os fixtures de `tests/fixtures/` não mudam. A
extração fica em `core/history.rs` e é coberta por testes contra os registros
reais deste repositório. Se ela virar regra, o lugar é o `docs/PROTOCOL.md`
primeiro.

## Consequências

### Positivas

- O histórico do trabalho cabe no painel; não é preciso abrir arquivo para saber
  o que cada tarefa entregou.
- Agora continua um painel de olhar, sem rolagem e com texto selecionável.
- Nenhuma superfície de escrita ou de execução é aberta.

### Negativas e custos assumidos

- O binário deixa de ser puramente passivo: passa a ter estado de tela e um
  modo de captura de mouse a restaurar em todo caminho de saída.

## Compliance

1. Nenhum módulo de navegação lê disco, escreve em `.specs/` ou em
   `.orchestration/`, ou cria processo; o módulo está na lista de `read_only.rs`.
2. A captura do mouse só está ligada com Histórico aberto, e é desligada ao voltar
   a Agora, ao sair e no panic; um teste em pty verifica a restauração.
3. Todo gesto do mouse tem uma tecla equivalente coberta por teste sem terminal.
4. `RelayState` e os fixtures (`app/relay-tui/tests/fixtures/`) não mudam por
   esta ADR.

## Notes

Spec de origem:
`.specs/20261002-002-relay-tui-navegacao-pelo-historico.md`.
