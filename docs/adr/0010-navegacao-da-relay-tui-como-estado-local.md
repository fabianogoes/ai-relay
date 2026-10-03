# ADR-0010 — relay-tui: navegação como estado local da tela

## Status

**Accepted** — 2026-10-03.

Emenda a decisão 1 da ADR-0009 ("só reage às teclas de saída") e os
não-objetivos de navegação da spec 20261002-001 ("navegar, selecionar" e
"changelog e texto de spec na TUI"). A ADR-0009 segue `Accepted` em tudo o mais,
inclusive na decisão 5 da ADR-0001 que ela preserva: o `relay-tui` nunca escreve
um registro.

## Contexto

A ADR-0009 entregou um painel passivo: desenha o estado de agora e só reage às
teclas de saída. Isso foi proposital ("Passivo agora"), para que a primeira
versão não carregasse superfície de interação. O uso real mostrou o limite: para
saber o que já foi feito (quais specs, quais itens de backlog, quais tarefas e o
que cada uma entregou) a pessoa abre os arquivos ou a UI web, embora o dado
esteja todo nos registros que o painel já lê.

O pedido é navegar por esse histórico no próprio painel, com teclado e com
mouse. Isso colide com a decisão 1 da ADR-0009 e traz um risco que ela não
tinha: a captura do mouse pelo terminal tira a seleção e a cópia de texto.

## Decisão

### 1. A navegação é estado local da tela, e continua somente leitura

Setas, `j`/`k`, `Enter`, `Esc`, `Backspace`, `Tab`/`t`, `r`, a roda e o clique
mudam só o que a tela mostra: a visão, o nível e a linha selecionada. Nenhuma
tecla ou clique escreve em `.specs/` ou em `.orchestration/`, abre editor ou
lança processo. `r` força uma releitura do workspace e nada mais. O módulo de
navegação não lê disco e entra na lista de módulos sem leitura do teste
`read_only.rs`; a decisão 5 da ADR-0001 fica intacta.

### 2. Uma segunda visão, Histórico, ao lado de Agora

**Agora** continua sendo a visão padrão ao abrir. **Histórico** aprofunda por
seleção em quatro níveis (specs, itens de backlog da spec, tarefas do item e
detalhe da tarefa). A tarefa concluída vem do changelog, porque o `TODO.md` é
esvaziado quando o item de backlog fecha: é o mesmo caminho da cascata spec,
backlog e changelog da UI web (ADR-0007, decisão 6). A ordem, as linhas, a
rolagem, a largura mínima e as teclas de cada nível são do design system, que
tem autoridade sobre a aparência; esta ADR não as repete.

### 3. O nome é Histórico, e não Trabalho

A UI web chama de Trabalho a visão em cascata (ADR-0007). "Trabalho" sugere o
que está em curso, que é o que Agora já mostra; a visão nova é o passado e o que
falta. O nome diverge da UI web de propósito, e é aqui que a divergência fica
registrada.

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

### 6. O parse novo fica só no Rust

A navegação exige extrair do changelog data, título e todos os campos (com as
linhas de continuação) e o título de cada spec. É extração de texto, sem regra
nova do protocolo: `RelayState` e a suite de conformidade não mudam, e o
`relay-core` em TypeScript não ganha paridade. O risco de divergência é baixo e
fica coberto por testes contra os registros reais deste repositório. Se essa
extração virar regra, estender o `relay-core` e a suite é uma decisão a tomar
então.

## Consequências

### Positivas

- O histórico do trabalho cabe no painel; não é preciso abrir arquivo nem a UI
  web para saber o que cada tarefa entregou.
- Agora continua um painel de olhar, sem rolagem e com texto selecionável.
- Nenhuma superfície de escrita ou de execução é aberta.

### Negativas e custos assumidos

- O binário deixa de ser puramente passivo: passa a ter estado de tela e um
  modo de captura de mouse a restaurar em todo caminho de saída.
- O nome Histórico diverge do Trabalho da UI web.
- A extração do changelog existe só no Rust; o `relay-core` em TS não a tem.

## Compliance

1. Nenhum módulo de navegação lê disco, escreve em `.specs/` ou em
   `.orchestration/`, ou cria processo; o módulo está na lista de `read_only.rs`.
2. A captura do mouse só está ligada com Histórico aberto, e é desligada ao voltar
   a Agora, ao sair e no panic; um teste em pty verifica a restauração.
3. Todo gesto do mouse tem uma tecla equivalente coberta por teste sem terminal.
4. `RelayState` e a suite de conformidade (`app/conformance/`) não mudam por esta
   ADR.

## Notes

Emenda a ADR-0009 (decisão 1) e a spec 20261002-001; a ADR-0007 deu a cascata que
Histórico reproduz no terminal. Spec de origem:
`.specs/20261002-002-relay-tui-navegacao-pelo-historico.md`.
