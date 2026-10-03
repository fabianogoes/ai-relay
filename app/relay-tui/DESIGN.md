# Design do `relay-tui`

**Status:** vigente — descreve o painel de terminal **implementado**. Decidido em
2026-10-02 (ADR-0003, spec `20261002-001`), com a visão Histórico (ADR-0004).

Este documento é a autoridade sobre a aparência do `relay-tui`: paleta, status,
estrutura e comportamento da tela. Uma mudança de cor, rótulo ou layout se faz
primeiro aqui e depois no código (`src/theme.rs` e `src/view/`); uma divergência entre este documento e a tela é
defeito de um dos dois e se resolve aqui primeiro.

O `relay-tui` é um painel passivo e estreito, pensado para dividir o terminal
com o harness. A paleta é própria do terminal porque um terminal não tem
`rgba`, fundo próprio nem fonte escolhida pela aplicação.

## Princípios

Toda decisão da tela decorre destas regras. Quando uma escolha de desenho
conflitar com um princípio, o princípio vence.

1. **O handoff é o produto.** O objeto central da tela é o handoff corrente e a
   próxima decisão humana. Specs, backlog e changelog são navegação.
2. **Deriva do disco e nunca escreve.** O estado vem dos cinco registros do
   protocolo; toda mutação passa por uma skill executada num harness.
3. **Estado nunca é comunicado só por cor.** Todo marcador de status tem rótulo
   textual além do tom.
4. **Não afirmar o que o protocolo não define.** Sem fila numerada, sem
   posição, sem barra de progresso percentual. O `TODO.md` não define ordem,
   dependência ou esforço; o backlog é *independentemente selecionável*.
5. **Sem superfície de execução.** O painel observa; não lança harness nem
   oferece tecla que execute algo.
6. **Dado antigo se declara antigo.** Enquanto os arquivos mudam, a tela mantém
   o último estado estável com o rótulo `atualizando`; nunca parece atual sem
   evidência.

**Contraste.** Toda cor que carrega informação (texto, IDs, caminhos, tempo)
atinge **AA** (4,5:1) sobre o fundo de referência. Cor abaixo disso só pode ser
decorativa.

## Fundo e contraste

A TUI **não pinta o fundo**: herda o do terminal, para casar com o painel do
harness ao lado. Por isso o contraste depende de um fundo que o Relay não
controla. Os valores abaixo foram medidos contra `#282c34` (fundo do One Dark)
e a paleta presume terminal escuro; tema claro fica fora da primeira versão.

## Paleta

| Papel | Valor | Contraste | Uso |
| --- | --- | --- | --- |
| `fg` | `#abb2bf` | 6,6:1 | texto primário |
| `meta` | `#9ba6b4` | 5,7:1 | texto secundário e metadados |
| `green` | `#98c379` | 6,9:1 | `in_progress`, `done`, sucesso |
| `blue` | `#61afef` | 5,9:1 | `ready`, `backlog`, item disponível |
| `yellow` | `#e5c07b` | 8,1:1 | `blocked`, atenção, dependência pendente |
| `red` | `#e06c75` | 4,4:1 | `inconsistent`, violação |
| `id` | `#ff75bf` | 5,7:1 | identificadores `B-NNN`, `T-NNN` (acento rosa do tema Charm) |
| `dim` | `#5c6370` | 2,3:1 | **só** bordas e traços decorativos — nunca texto |
| `bar_empty` | `#3e4451` | 1,4:1 | **só** segmento vazio da barra do TODO — nunca texto |

Os nove valores base vêm do One Dark, o tema padrão do `ai-usagebar`; `meta` e
`id` são decisões deste documento, e o laranja do One Dark fica sem uso.

Duas regras que a medição impôs:

- **`dim` não carrega informação.** Com 2,3:1 ele não atinge AA (ver
  Contraste, acima). O One Dark o usa para texto secundário; aqui esse papel é do
  `meta`, que o One Dark não tem em AA (`#828997` dá 4,0:1).
- **Lacuna aceita: `red` com 4,4:1** está abaixo de 4,5:1 sobre `#282c34`, e
  passa em fundos mais escuros (5,2:1 sobre `#1e1e1e`). Mitigação: o vermelho
  nunca está sozinho — vai sempre em negrito, com o glifo e o rótulo
  "Inconsistente"; o corpo das violações usa `fg`.

## Status

| Status | Rótulo | Tom |
| --- | --- | --- |
| `in_progress` | Em andamento | green |
| `blocked` | Bloqueado | yellow |
| `inconsistent` | Inconsistente | **red** |
| `ready` | Pronto | blue |
| `backlog` | A escolher | blue |
| `done` | Concluído | green |
| `idle` | Sem trabalho | `meta` |

`inconsistent` tem um tom só dele, separado do amarelo de `blocked`, porque é
o estado que trava o resto da tela. Tom e rótulo textual são obrigatórios
(princípio 3).

## Estrutura

- **Cartões** com borda arredondada (`╭ ╮ ╰ ╯`) e o título na própria borda:
  cabeçalho, Handoff, TODO, Spec atual, Specs pendentes e rodapé. A borda é `dim`; a do Handoff herda o
  tom do status. Em `inconsistent`, um cartão de violações (`check` e `detail`)
  substitui o conteúdo.
- **Marcadores do TODO** acompanham o protocolo e sempre trazem texto ao lado:
  `✓` concluído, `●` em andamento, `○` disponível, `◌` indisponível com
  "após T-NNN". Os glifos são Unicode; a primeira versão não define fallback
  ASCII.
- **Barra do TODO** com um segmento por item, no tom do item. É contagem e
  nunca porcentagem (princípio 4).
- **Frescor em texto:** `atualizado` ou `atualizando`, no cabeçalho.
- **Largura de referência: 58 colunas.** Sem rolagem: faltando altura, o TODO
  trunca em "+N itens" e o backlog (Spec atual e Specs pendentes) reduz-se a uma
  linha de contagem; abaixo de
  cerca de 40 colunas aparecem só o cabeçalho e o status.
- **Teclas:** em Agora, `q` e `Ctrl-C` saem na hora e `Esc` pergunta antes (ver
  "Confirmar a saída"); `Tab` abre a visão Histórico
  e `r` recarrega (ADR-0004; ver "Visão Histórico"). O rodapé mostra só as teclas
  válidas na tela atual.

## View

A tela é função pura do estado derivado, do frescor e do relógio: nenhum
cálculo do protocolo (princípio 2; `app/AGENTS.md`). De cima para baixo:

1. **Cabeçalho**, uma linha: o selo `relay` (fundo `green`, texto `#282c34`), o
   caminho do workspace em `meta` e, à direita, o frescor — `● atualizado` ou
   `● atualizando`. O ponto é `green` ou `yellow`; a palavra, em `meta`, é quem
   informa. O selo é a marca do aplicativo, constante, e não um status.
2. **Cartão Handoff**, quando há handoff. Título `Handoff`; à direita o status
   em negrito no seu tom (`● Em andamento`, `● Bloqueado`); a borda herda o tom.
   Primeira linha: `B-NNN · T-NNN · <harness> · há 4 min`, com os IDs em `id` e o
   resto em `meta`. Depois `Objetivo` e `Próximo`, rótulos em `meta` e texto em
   `fg`, quebrado com recuo alinhado ao texto. Em `blocked` entra também
   `Bloqueio`, com o `Context` do handoff, que traz o bloqueio e a condição de
   retomada. O teto de linhas de cada campo acompanha a altura: havendo espaço para o TODO e
o Backlog inteiros, o Handoff mostra os campos quase por inteiro (até 10 linhas,
12 no `Bloqueio`); sem essa folga, 3 linhas (4 no `Bloqueio`); e, se nem isso
couber, a versão compacta abaixo. O que passa do teto termina em `…`.
3. **Cartão TODO**, quando há TODO. Título `TODO`; à direita `feitos/total`. A
   barra tem um segmento por item, no tom do estado do item (`green` feito ou em
   andamento, `blue` disponível, `bar_empty` indisponível) — é contagem, nunca
   porcentagem. Cada item: marcador, ID em `id` e texto; feito em `meta`, em
   andamento em `fg` negrito, indisponível com `após T-NNN` em `yellow`.
4. **Spec atual e Specs pendentes**, no lugar do antigo cartão Backlog, quando há
   backlog (ver "Agrupado por spec" abaixo). Em `backlog` (A escolher), onde não
   há TODO nem handoff, a linha `● A escolher  Sem handoff ativo` (como a de
   `ready`) fica no lugar do Handoff e os dois cartões no lugar do cartão `A
   escolher`.
5. **Próximo passo**, uma linha logo acima do rodapé (ver "Próximo passo"
   abaixo).
6. **Rodapé**, um cartão de três linhas (borda `dim`) com `Tab histórico · r
   recarregar · q sair`, as teclas em negrito `fg` e o resto em `meta`, separado do
   corpo por uma linha em branco. Abaixo de 10 linhas de altura (sem a linha em
   branco do topo também) volta a ser uma linha só, como antes. Quando não cabe na largura útil dos cartões
   (a largura menos quatro), saem indicações inteiras, `r recarregar` primeiro e
   depois `Tab`; `q sair` fica sempre. Em 40 colunas, `Tab histórico · q sair`.

**Próximo passo.** Uma linha, e não um cartão: um cartão competiria com o
Handoff, que é o objeto central da tela (princípio 1). Ela é função só do estado
já carregado (nada de relógio) e **sugere sem executar**: a TUI não acrescenta
tecla nem interação. A skill vai em texto e em negrito `fg`; os IDs em `id`; o
resto da frase em `meta`; nada depende só da cor. Ela começa na mesma coluna do conteúdo dos cartões e não
passa da largura útil (a largura menos quatro): quando não cabe, o título entre
parênteses encurta primeiro (e some se não sobrar espaço), de modo que a skill a
chamar nunca é o que se corta. As frases longas têm uma forma curta, usada quando a
longa não cabe nem sem o título (por exemplo `Próximo item: B-NNN (título).
Comece com relay-session.`, `Nenhum item disponível: relay-continue.`, `Registros
em conflito: relay-status e relay-continue.`). O primeiro caso que casar vale:

| Caso | Condição | Frase |
| --- | --- | --- |
| não é workspace | diretório sem `.orchestration/` | Instale o protocolo com `relay-setup`. |
| `inconsistent` | violações | Os registros se contradizem: `relay-status` mostra o diagnóstico e `relay-continue` pode propor o reparo. |
| `in_progress` | handoff `in_progress` | Retome `T-NNN` de `B-NNN` com `relay-session`. |
| `blocked` com handoff | handoff `blocked` | Resolva o bloqueio de `T-NNN` (ver Handoff) e retome com `relay-session`. |
| `blocked` sem handoff | TODO sem item disponível | Nenhuma subtarefa disponível em `B-NNN`: `relay-continue` mostra o bloqueio e as opções. |
| `ready` | TODO com item disponível | Comece `T-NNN` (título) com `relay-session`. |
| `done` com TODO | TODO todo `[x]`, sem handoff, item ainda aberto no backlog | Subtarefas de `B-NNN` concluídas: feche o item com `relay-session`. |
| `backlog` com disponível | há entrada de backlog disponível | Próximo item disponível: `B-NNN` (título). Comece com `relay-session`. |
| `backlog` sem disponível | nenhuma entrada disponível | Nenhum item disponível: resolva os bloqueios ou dependências do backlog; `relay-continue` mostra as opções. |
| `done` | qualquer outro `done` | Tudo concluído. Para uma nova ideia: `relay-spec`. |
| `idle` | sem backlog, TODO nem handoff | Nada em andamento. Para começar: `relay-spec`. |

"Próximo item disponível" é a recomendação padrão do protocolo (a primeira
entrada disponível em ordem textual), sem dizer que é a de maior prioridade. Em
`relay-tui` a lógica fica em `suggest`, fora do `core`; a view só monta a frase.
A linha não existe na visão Histórico.

**Estados sem handoff.** `ready` mostra, no lugar do Handoff, uma linha que
leva o status — `● Pronto` em negrito `blue` — e `Sem handoff ativo` em `meta`,
sobre o cartão TODO. `done` mostra um cartão
`Concluído` em `green` com a contagem do backlog. `idle` mostra um cartão
`Sem trabalho` com `Nenhum backlog, TODO ou handoff neste workspace.`. Um
diretório sem `.orchestration/` mostra `Não é um workspace Relay`, com o caminho
observado, e continua vigiando.

**Tamanho.** Os cartões de um só assunto (`Sem trabalho`, `Concluído`, `A
escolher`, `Inconsistente`, `Não é um workspace Relay`) têm a altura do próprio
conteúdo, não a da tela; sobra espaço vazio embaixo, nunca moldura vazia.

**Inconsistente.** O cartão `Inconsistente` (borda e título em `red`) substitui
os demais. Cada violação: o `check` em negrito `red`, o `detail` em `fg` quebrado
e os `records` em `meta`, separados por ` · `. O corpo nunca é vermelho (ver a
lacuna aceita acima).

**Tempo relativo**, do `Updated` do handoff e do relógio injetado: `agora` (menos
de um minuto), `há N min`, `há N h` (menos de um dia), `há N d`. Um `Updated` no
futuro, por relógio desajustado, também é `agora`.

**Agrupado por spec.** O backlog aparece em dois cartões, na ordem do protocolo
e sem recalcular nada (o `available` e o marcador vêm do core):

- **Spec atual**: na borda, o id `AAAAMMDD-NNN` e o título da spec (o mesmo do
  nível de specs de Histórico, cortado com `…` antes do lado direito); à direita
  `em curso` (negrito `green`) ou `a seguir` (negrito `blue`) e `feitos/total`.
  A spec atual é a do item ativo do TODO; sem item ativo (estado `backlog`), a do
  primeiro item disponível em ordem textual, a recomendação padrão do protocolo, e
  o cartão diz `a seguir`, sem afirmar prioridade. Dentro, os itens ainda não
  feitos da spec, na ordem textual, com o marcador à esquerda e a palavra à direita
  (`em curso`, `disponível`, `aguardando · após B-NNN` em `yellow`, `bloqueado`);
  os feitos entram só na contagem. Sem item que ancore a spec (backlog sem item
  disponível, ou item ativo sem spec válida) o cartão não aparece.
- **Specs pendentes**: uma linha por spec, além da atual, que tem ao menos um item
  não feito, na ordem em que aparecem pela primeira vez no `BACKLOG.md`: id,
  título e `feitos/total`. Os itens sem spec válida formam a linha **Sem spec**,
  por último. Sem outra spec pendente o cartão não aparece.

**Altura.** Não há rolagem. O cabeçalho tem prioridade. Faltando altura, cede
primeiro o backlog: Specs pendentes vira uma linha (`N specs pendentes`), depois
Spec atual corta em `+N itens` (sempre com ao menos um item: abaixo disso não há
cartão cortado) e, por fim, os dois viram a linha de contagem do Backlog (`Backlog
32/37 · 1 em curso · 3 disponíveis`, omitindo o que não cabe inteiro); depois a linha de
próximo passo, que some antes de o Handoff se compactar e, portanto, sempre antes
de o TODO cortar; depois o Handoff, que
perde o espaço extra e, se preciso, se compacta — sem a linha em branco e com
uma linha por campo — para deixar o mínimo do TODO (moldura, barra e uma linha); por fim o TODO corta em `+N itens`,
e a linha de corte também mostra o ID do item em andamento, se ele ficou fora.
Abaixo de 6 linhas, só cabeçalho e rodapé.

**Plural.** As palavras concordam com o número (`1 feito`, `2 feitos`; `1
disponível`, `3 disponíveis`; `+1 item`, `+4 itens`; `+1 violação`).

**Largura.** A referência é de 58 colunas. Texto longo quebra (campos do handoff)
ou termina em `…` (itens, caminho); nunca estoura a moldura. Abaixo de 40
colunas só aparecem o cabeçalho e o status em uma linha (`● Em andamento  B-034`),
sem cartões e sem a linha de próximo passo. A linha de próximo passo, quando não
cabe, termina em `…`.

## Visão Histórico

Segunda visão do `relay-tui`, ao lado de **Agora**, que continua sendo a padrão
ao abrir (ADR-0004). O nome é Histórico, e não Trabalho, porque Trabalho
sugere o que está em curso, que é o que Agora já mostra; Histórico é o passado
e o que falta. É estado local da tela e continua somente leitura: nenhuma
tecla nem clique escreve num registro.

**Níveis.** Quatro, cada um aprofundando o anterior: **specs**, **itens de
backlog** da spec, **tarefas** do item e **detalhe** da tarefa. `Tab` (ou `t`)
alterna com Agora de qualquer nível, e voltar ao Histórico reabre o nível e a
seleção em que a pessoa estava.

**Ordem e linhas.** Cada linha de lista ocupa uma linha e o que não cabe termina
em `…`. Contagens são `feitos/total`, nunca porcentagem.

| Nível | Ordem | Linha |
| --- | --- | --- |
| Specs | arquivos de `.specs/`, do mais recente ao mais antigo (nome decrescente) | id `AAAAMMDD-NNN` em `id`, título e `feitos/total` dos itens (`0/0` sem itens); uma última linha **Sem spec** reúne os itens cuja `spec:` falta ou não é um arquivo de `.specs/` |
| Itens de backlog | ordem textual do `BACKLOG.md` | id, texto (sem as anotações `spec:`/`needs:`) e o estado, com o mesmo marcador e palavra do cartão Backlog de Agora |
| Tarefas | registros do changelog cujo `Backlog` é o item, na ordem textual; um `T-NNN` repetido (correção append-only) é uma linha própria | id, título e data do registro; se o item é o do TODO atual, os itens do TODO ainda sem registro vêm depois, com o marcador do TODO e a palavra `sem registro` |
| Detalhe | — | título inteiro e os campos do registro (Backlog, Spec, Result, Evidence, Criteria, Decisions), com as linhas de continuação juntadas ao campo; campo ausente é omitido, nunca inventado; tarefa sem registro mostra id, texto, marcador e `Sem registro no changelog ainda.` |

O título de uma spec é o texto após `AAAAMMDD-NNN - ` no primeiro `# ` do
arquivo; sem esse formato, o texto inteiro do `# `; sem `# `, o nome do arquivo.

**Rolagem.** Só aqui (Agora continua sem rolagem). A lista rola para manter a
seleção visível e diz, em `meta`, quantas linhas há acima e abaixo; o detalhe
rola com setas, `j`/`k`, `PgUp`/`PgDn` e a roda. O detalhe quebra o texto pela
largura e nunca corta.

**Largura.** A referência é de 58 colunas e a visão se desenha também em 40.
Abaixo de 40, mostra só o cabeçalho, a linha `Histórico precisa de 40 colunas`
e o rodapé; as teclas continuam valendo. Sem `.orchestration/`, a lista fica
vazia com `Nenhuma spec em .specs/`. A visão funciona também em `inconsistent`.

**Teclas e cliques.** Todo gesto do mouse tem equivalente de teclado.

| Gesto | Efeito |
| --- | --- |
| `Enter` ou clique numa linha | abre o nível seguinte; no detalhe `Enter` não faz nada |
| `↑` `↓`, `j` `k`, roda | movem a seleção, sem passar dos limites |
| `Esc` ou `Backspace` | voltam um nível; no nível de specs, voltam a Agora |
| `Tab` ou `t` | alternam Agora e Histórico |
| `r` | força a releitura do workspace (em Agora e em Histórico), sem escrever nada |
| `q`, `Ctrl-C` | saem de qualquer visão, na hora (em Agora `Esc` também sai, mas depois de confirmar) |

Clique fora de uma linha (cabeçalho, rodapé, área vazia) é ignorado; voltar um
nível pelo mouse não existe. A captura do mouse fica ligada só enquanto Histórico
está aberto, para que Agora continue permitindo selecionar e copiar texto.

**Atualização.** Quando os registros mudam, as listas são refeitas e a seleção
é mantida pelo id (spec, `B-NNN`, posição do registro); se o item sumiu, vai
para a linha mais próxima e, se o nível inteiro sumiu, sobe até o primeiro nível
que existe.

**Rodapé**, em `meta`, só com as teclas válidas na tela:

- Agora: `Tab histórico · r recarregar · q sair`.
- Histórico, nas listas: `↑↓ mover · Enter abrir · Esc voltar · Tab agora · r
  recarregar · q sair`.
- Histórico, no detalhe: `↑↓ rolar · Esc voltar · Tab agora · r recarregar · q
  sair`.

Quando não cabe, saem indicações inteiras, nunca meia, da menos para a mais
importante: `r recarregar`, `↑↓`, `Enter abrir`, `Esc voltar`, `Tab`; `q sair`
fica sempre. Em 40 colunas, Agora mostra `Tab histórico · q sair`.

## Confirmar a saída

`Esc` em Agora não sai sozinho (um toque perdido encerraria o painel): o rodapé
é trocado pela pergunta `Sair?` (negrito `yellow`) seguida das respostas em `meta`,
`Esc, Enter ou y confirmam · outra tecla cancela`. `Esc`, `Enter` e `y` confirmam;
qualquer outra tecla cancela, o rodapé volta e a tecla é consumida (não faz o que
faria). `q` e `Ctrl-C` são gestos explícitos e saem na hora. Quando a frase não
cabe na largura útil dos cartões (a largura menos quatro), usa-se a mais longa que
cabe: sem `· outra tecla cancela`, depois `Esc ou y confirmam`, `y confirma` e
só `y`; as teclas de resposta nunca são cortadas ao meio. Em Histórico não há
pergunta: `Esc` volta.
