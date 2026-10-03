# Design System do Relay

**Data:** 2026-09-06 · revisado em 2026-09-11 e 2026-10-02 (seção 10, meio terminal)
**Autor:** Claude Code
**Status:** vigente — descreve a interface **implementada**

Este documento é a **fonte de autoridade** para toda a interface do Relay. A
proposta ([`ui-proposal.md`](ui-proposal.md)) e os protótipos
([`prototypes/`](prototypes/)) permanecem no repositório como referência visual
e histórica, mas o que vale para implementação é o que está aqui. Uma mudança de
token ou de componente se faz primeiro neste documento, depois no código.

Na revisão de 2026-09-11 esse "vale para implementação" deixou de ser
aspiração: o observador read-only foi construído, aberto num browser contra um
repositório Relay real e ajustado contra este documento até bater, e cada
divergência encontrada virou decisão aqui **antes** do código. As seções 5 e 6
descrevem agora o que está em pé — HandoffCard, lista de subtarefas, as três
colunas da aba Trabalho, o cartão de backlog, o registro de changelog e o modal
de tarefas. Onde um protótipo mostra outra coisa, o protótipo é o histórico e
este documento é o estado oficial.

A decisão de arquitetura que emoldura a UI está na
[`../adr/0001-arquitetura-inicial-da-ui.md`](../adr/0001-arquitetura-inicial-da-ui.md).
Este documento não repete a arquitetura; ele padroniza o desenho da superfície.

## Índice desta pasta

| Arquivo | O que é | Como usar |
| --- | --- | --- |
| `README.md` (este) | fonte de verdade do design system: tokens, componentes e regras | ler antes de qualquer mudança de interface |
| [`ui-proposal.md`](ui-proposal.md) | análise exploratória e arquitetura da UI (raciocínio e alternativas descartadas) | ler só quando a decisão de desenho não estiver na ADR |
| [`design-system.html`](design-system.html) | exemplo visual **derivado** deste README | abrir no browser; nunca ler por agente. Regenerado em 2026-09-11 a partir deste documento, com todos os tokens **web** e os componentes da seção 6. Não cobre a seção 10 (meio terminal): ali não há CSS a derivar |
| [`prototypes/`](prototypes/) | protótipos históricos, ~350 KB cada | **nunca abrir num agente**; só referência visual no browser. Superados pela interface implementada — servem para ver de onde veio uma decisão, nunca para decidir |
| [`audit-2026-09-11.md`](audit-2026-09-11.md) | auditoria final por componente (A-015 da spec 013): contraste, cor+rótulo, foco, alvo, hierarquia, semântica, responsividade | ler para saber o que já foi verificado e o que ficou registrado como lacuna aceita, antes de reabrir uma auditoria do zero |

> **Nunca leia os `.html`** (`design-system.html` e `prototypes/*.html`) num
> agente. O `design-system.html` duplica tokens deste README em CSS e o que há
> nos protótipos já foi extraído: o que foi **decidido** está na ADR-0001; o que
> foi **observado** está em `ui-proposal.md`. Ao mudar um token aqui, atualize o
> `design-system.html` junto — ele é derivado, não fonte.

---

## 1. Princípios

Toda decisão de interface decorre destas regras. Quando uma escolha de desenho
conflitar com um princípio, o princípio vence.

1. **O handoff é o produto; o backlog é navegação.** O objeto central de toda
   tela é o handoff corrente e a próxima decisão humana. Specs, backlog e
   changelog são trabalho de outra visão.
2. **A UI deriva do disco e nunca escreve.** O estado vem dos cinco registros do
   protocolo (Canal B); o stdout do harness nunca é parseado. A aplicação lê,
   deriva e lança — toda mutação passa por uma skill executada em um harness
   (ADR-0001, ponto 5).
3. **Três telas, não seis status.** Os seis status são leitura de máquina. Para
   quem decide existem três situações: **Retomar**, **Escolher** e **Reparar** —
   cada uma com uma ação primária única.
4. **"Gravado em disco" fecha toda execução.** Toda execução termina mostrando
   o que mudou no disco, arquivo por arquivo. Isso torna visível a tese do
   produto: a memória está no disco, não no chat.
5. **Preflight honesto.** O preflight mostra o `argv` elemento por elemento,
   nunca uma string com aspas montada para parecer comando de shell. Preflight
   que mente é pior que preflight nenhum.
6. **Preferência não é autorização.** A escolha do harness pode ser local ao
   workspace; não é consentimento para executar. O preflight é obrigatório em
   toda execução.
7. **Estado nunca é comunicado só por cor.** Todo marcador de status tem rótulo
   textual além do tom.
8. **Não afirmar o que o protocolo não define.** Sem fila numerada, sem rótulo
   `FIFO`, sem posição, sem barra de progresso percentual, sem pista com
   dependências. O `TODO.md` não define ordem, dependência ou esforço; o backlog
   é *independentemente selecionável*.
9. **Read-only é o produto padrão.** A abertura normal desliga a rota de
   lançamento por completo e apresenta o observador do workflow. A execução
   integrada permanece como opt-in experimental por `--exec`; read-only é o
   mesmo produto com a superfície de execução removida — não um modo capado.
10. **Dado antigo se declara antigo.** Durante uma transição entre arquivos, a
    UI mantém o último snapshot estável com o rótulo "Atualizando". Quando a
    conexão cai, mantém o conteúdo para consulta com o rótulo "Desatualizado"
    até receber um novo snapshot; nunca parece atual sem evidência. Declarar
    não basta: nenhum botão que lança execução (▶ Começar, ▶ Retomar, em
    qualquer uma das três telas) fica clicável enquanto o rótulo é
    "Desatualizado" — o dado que a ação usaria (handoff, harness, ID) pode já
    não ser real. O botão continua visível, só ganha `disabled` e um texto
    curto explicando por quê (nunca cinza silencioso); volta a habilitar
    sozinho quando um snapshot novo chega. Vale só para "Desatualizado"
    (conexão caiu de fato) — "Atualizando" é uma transição breve entre
    arquivos com dado ainda válido, não desabilita nada.

---

## 2. Tokens de cor

Tema escuro. Os valores abaixo são os adotados no protótipo v2 e aqui fixados,
com uma exceção registrada logo após a tabela de acentos: os tons de identidade
de harness, em que este documento diverge do protótipo de propósito.

| Token | Valor | Uso |
| --- | --- | --- |
| `--bg` | `#0b0d10` | fundo da aplicação |
| `--bg-deep` | `#070a0d` | fundo de modais e sobreposições; também a superfície recuada quando um painel precisa conter outro (as calhas da aba Trabalho, as caixas do HandoffCard) |
| `--panel` | `#0f1318` | cartões e painéis |
| `--panel-2` | `#151a21` | painéis elevados, tabelas |
| `--raise` | `#1a2028` | estado de hover e superfícies destacadas |
| `--ink` | `#e9edf3` | texto primário |
| `--ink-2` | `#c6cfda` | texto secundário |
| `--meta` | `#9ba6b4` | metadados: caminhos, timestamps, IDs |
| `--line` | `rgba(255,255,255,.10)` | bordas padrão |
| `--line-2` | `rgba(255,255,255,.16)` | bordas de destaque e hover |
| `--scrim` | `rgba(0,0,0,.55)` | véu que separa uma sobreposição (modal, seletor) do conteúdo de trás |

Acentos semânticos — cada um com três variantes (tinta, fundo suave, linha):

| Token | Valor | Uso |
| --- | --- | --- |
| `--green` / `--green-soft` / `--green-line` | `#5fe3b3` / `rgba(95,227,179,.10)` / `rgba(95,227,179,.34)` | `in_progress`, sucesso |
| `--amber` / `--amber-soft` / `--amber-line` | `#f7cd7a` / `rgba(247,205,122,.10)` / `rgba(247,205,122,.36)` | `blocked`, `inconsistent`, atenção |
| `--blue` / `--blue-soft` / `--blue-line` | `#93baff` / `rgba(147,186,255,.10)` / `rgba(147,186,255,.32)` | `ready`, `backlog`, seleção |
| `--purple` / `--purple-soft` / `--purple-line` | `#cfaaff` / `rgba(207,170,255,.10)` / `rgba(207,170,255,.32)` | identidade de harness: Codex |
| `--orange` / `--orange-soft` / `--orange-line` | `#f5a878` / `rgba(245,168,120,.10)` / `rgba(245,168,120,.34)` | identidade de harness: Claude Code |
| `--on-green` | `#05231a` | texto sobre fundo `--green` sólido |

> **Divergência conhecida do protótipo v2, medida na navegação.** Lá o selo do
> Codex usa `#93baff` — o próprio `--blue`, que a seção 3 reserva para
> `ready`/`backlog`/seleção — e o do Claude Code usa `#e0865f`, valor que não
> existe como token; `--orange` sequer está declarado no protótipo. A decisão
> da seção 6 ("Lista de harnesses") vale contra isso, e a implementação segue
> este documento. A justificativa de que o roxo "acompanha o protótipo" era
> falsa quanto ao fato; a decisão continua de pé pelo argumento que importa —
> `--blue` já carrega significado de status e não pode carregar identidade
> também, que é exatamente a colisão que o protótipo produz.

### Contraste

Toda cor que carrega informação (texto, IDs, caminhos, timestamps) deve atingir
**AA** (4,5:1 para corpo, 3:1 para texto grande). O `--meta #9ba6b4` sobre
`#0b0d10` atinge ≈7:1 e é o mínimo permitido para metadados.

> **Proibido:** os cinzas `#4a5462` (≈2,4:1) e `#5d6675` (≈3,2:1) sobre o fundo
> atual, que não atingem AA e carregam informação real. Não reutilizá-los do
> protótipo v1.

---

## 3. Mapeamento de status → tom

| Status | Rótulo na interface | Tom | Nota |
| --- | --- | --- | --- |
| `in_progress` | Em andamento | green | ação primária: retomar |
| `blocked` | Bloqueado | amber | requer `Context` com o bloqueio e a condição de retomada |
| `inconsistent` | Inconsistente | amber | tela de reparo dura; restringe o resto |
| `ready` | Pronto | blue | selecionável |
| `backlog` | A escolher | blue | selecionável |
| `done` | Concluído | green | conclusão |
| `idle` | Sem trabalho | meta (fundo transparente, linha `--line`) | sem trabalho ativo |

Todo marcador de status combina **tom + rótulo textual** — essa é a regra
inegociável, e vale em qualquer lugar que exiba status.

A forma padrão é o `StatusPill` (seção 6), e é a única permitida quando o
status aparece **solto**, fora de um objeto que já carregue o tom. Duas
superfícies exibem status sem ele, por decisão registrada na seção 6, e ambas
mantêm tom **e** texto: a **lista de subtarefas**, onde o rótulo acompanha o
marcador do protocolo, e o **cartão de backlog**, que já é tonalizado por
inteiro — ali um chip dentro de uma caixa colorida repetiria a mesma
informação três vezes. Nenhuma outra superfície inventa uma terceira forma.

---

## 4. Tipografia

Famílias: **Space Grotesk** (sans) e **JetBrains Mono** (mono), carregadas via
Google Fonts. Mono para tudo que vem do disco: caminhos, IDs, timestamps,
comandos, `argv`.

| Papel | Fonte | Tamanho / altura de linha | Peso |
| --- | --- | --- | --- |
| Display (cabeçalho de tela) | sans | 26 / 1,2 | 600 |
| Título de seção | sans | 22 / 1,25 | 600 |
| Subtítulo | sans | 20 / 1,35 | 600 |
| Título operacional | sans | 16 / 1,4 | 600 |
| Título de cartão | sans | 15 / 1,4 | 600 |
| Corpo | sans | 14 / 1,5 | 400 |
| Corpo de destaque | sans | 14 / 1,5 | 500 |
| Corpo secundário | sans | 13 / 1,5 | 400 |
| Metadados | mono | 12 | 400 / 500 |
| Dados em linha | mono | 12 / 1,5 | 400 / 500 |
| Comando / `argv` | mono | 13 / 1,6 | 400 |

Regras:

- Corpo em **14 px** no mínimo; metadados em **12 px** no mínimo; títulos
  operacionais em **16 px**.
- Nunca abaixo de 12 px em texto que carrega informação.
- `letter-spacing` negativo (`-.2px` a `-.4px`) apenas em títulos grandes.
- **Metadados** e **Dados em linha** têm a mesma especificação numérica (mono,
  12 px, peso 400/500) e compartilham um único conjunto de tokens de
  tipografia no código; a tabela lista os dois papéis porque descrevem usos
  diferentes, não porque exigem valores diferentes.

---

## 5. Layout

### Grade e container

- Container de aplicação com `max-width: 1280px`, centralizado, com respiro
  lateral (`--space-8`).
- Layout em flexbox; grade `flex:1` para painéis iguais.
- Escala de espaçamento (base 4):

| Token | Valor |
| --- | --- |
| `--space-1` | 4px |
| `--space-2` | 8px |
| `--space-3` | 12px |
| `--space-4` | 16px |
| `--space-5` | 20px |
| `--space-6` | 24px |
| `--space-8` | 32px |

### Altura de painel com rolagem interna

| Token | Valor | Uso |
| --- | --- | --- |
| `--handoff-context-max-height` | `22vh` | teto das duas caixas do HandoffCard ("Próximo passo" e "Contexto deixado") antes de rolarem internamente; é o mesmo valor nas duas, e é o que garante que tenham sempre o mesmo tamanho |

Relativo à viewport, não em pixel fixo, porque a regra que ele serve — a tela
**Retomar** cabe inteira sem rolagem de página (Header, HandoffCard e lista de
subtarefas) — depende da altura real da janela, não de um valor de desenho
fixo. A lista de subtarefas usa o espaço flexível restante (`flex: 1`) com a
mesma rolagem interna, em vez de um segundo valor fixo.

### Viewport estreita e safe areas

Em **760px ou menos**, a interface deixa de tentar preservar as grades largas:
o Header quebra em linhas, as duas caixas do HandoffCard e as três colunas da
aba Trabalho passam a uma coluna, e terminal + registro de disco do modo de
execução empilham verticalmente. A leitura segue a mesma ordem do desktop;
não existe uma segunda navegação ou uma versão móvel com menos informação.

O container da aplicação, overlays e modo de execução respeitam
`safe-area-inset-top`, `safe-area-inset-right`, `safe-area-inset-bottom` e
`safe-area-inset-left`, sempre conservando ao menos o respiro da escala de
espaçamento. Caminhos longos truncam no Header e quebram dentro de cartões;
ações quebram em linhas em vez de sair da viewport. Modais ocupam apenas a
altura dinâmica disponível e rolam internamente. Na aba Trabalho, cada lista
mantém rolagem própria com teto relativo à viewport para que uma coluna longa
não esconda as demais.

O breakpoint é regra de layout, não token: custom properties não participam de
media queries. Acima de 760px permanecem a grade de três colunas e as duas
colunas do HandoffCard; abaixo dele, `100dvh` é a referência de altura para
evitar que as barras móveis do navegador ocultem controles.

### As três telas

| Tela | Quando | Ação primária única |
| --- | --- | --- |
| **Retomar** | `in_progress`, `blocked` | ▶ Retomar T-002 no Codex |
| **Escolher** | `ready`, `backlog`, `idle` | escolher o que começar |
| **Reparar** | `inconsistent` | mostrar o conflito e a correção determinística |

A tela **Reparar** é dura: quando o protocolo determina que ninguém avança com
estado inconsistente, a interface **desabilita** o resto — não apenas pinta um
aviso. Restrição por construção vale mais que restrição por texto.

A segunda visão (aba **Trabalho**, ao lado de **Agora**) é um painel de três
colunas em cascata, não uma sequência de telas nem três painéis independentes:
1 spec selecionada → N **cartões de backlog** (seção 6); 1 cartão de backlog
selecionado → N **cartões de changelog** (seção 6), filtrados pelo `backlogId`
daquele cartão.

Cada coluna abre com um **rótulo mono em versal**, e um metadado à direita
dele quando houver: `SPECS` (com a ação "+ nova spec" quando execução estiver
habilitada), `BACKLOG DA SPEC` · `spec {id}`, `CHANGELOG` · `N registros`. O
rótulo usa o tom que a seção 3 já dá àquele conteúdo — `--blue` para Specs e
para Backlog (o tom de `backlog`/seleção), `--green` para Changelog (o tom de
`done`, que é o que um registro de changelog é). Nenhum desses tons carrega
estado novo: o estado continua no rótulo de status de cada cartão, sempre com
texto.

As três colunas ordenam **do mais recente para o mais antigo**, a mesma regra
da lista de subtarefas da aba Agora: a spec criada por último no topo, o
backlog na ordem inversa do `BACKLOG.md`, o changelog já nasce assim. A ordem
textual desses arquivos não carrega prioridade nem sequência (é o que o
protocolo diz em "Dependencies"), então invertê-la para exibir não desmente
nada — e põe no topo o que se está olhando agora. O item selecionado por
padrão é o **primeiro exibido**, ou seja, o mais recente; a regra
determinística de "primeiro da ordem textual" continua sendo a do protocolo
para *escolher trabalho*, que é outra coisa: aqui a seleção é um cursor de
leitura, não uma decisão de execução.

A visão inteira é **um cartão**, e cada coluna é **outro cartão dentro dele**
— três superfícies aninhadas, resolvidas pela escala de fundo da seção 2 e
não por sombra: `--panel` no cartão externo, `--bg-deep` na coluna (a mais
escura da escala, lida como calha recuada) e `--panel-2` nos cartões de spec e
de backlog, que assim ficam elevados sobre a calha. O registro de changelog
não tem fundo próprio: é só a régua de acento, e a calha já o separa do resto.
Sem esse degrau de três valores o aninhamento vira moldura dentro de moldura
sem informação. A rolagem entre colunas é independente, dentro do cartão
externo; nenhuma depende de rolagem de página.

A coluna do meio mostra **apenas os cartões de backlog** da spec selecionada.
O texto integral da spec não é reproduzido ali: ele é a intenção de origem,
não o trabalho selecionável, e um documento inteiro em mono afogava os
cartões que são o objeto real da coluna (ADR-0007, decisão 3, revisada). O
caminho do arquivo continua visível no cartão de spec, que é a resposta para
"onde leio isso por inteiro".

Changelog é **estruturado**: `relay-core` já possuía o parser de registros
(usado nas verificações de integridade) e agora o expõe também como dado
derivado (data, subtarefa, título, backlog, evidência) por uma rota dedicada
do `relay-host`; a UI nunca reimplementa esse parser — ela só filtra e
renderiza a lista já estruturada que recebe. O invariante continua o mesmo
(só `relay-core` interpreta a gramática do protocolo); o que mudou foi a
decisão específica de manter changelog como texto cru, registrada na ADR-0007
e revisada ali. O handoff permanece o objeto central só da tela principal
(aba **Agora**).

A tela **Escolher** não é um aviso de vazio. Ela lista as tarefas de backlog
disponíveis, uma por linha, cada uma com o próprio ID e o caminho da spec em
mono, precedidas do texto que diz **por que** a escolha é do usuário — as
tarefas do backlog são independentes entre si, e `needs` é a única dependência
real — em vez de apenas constatar que o handoff está vazio.

Com execução habilitada, cada linha ganha a própria ação primária nomeando o
harness ("▶ Começar no {harness}") e a tela fecha com um cartão para
especificar uma ideia nova ("Iniciar entrevista"); não há ação primária única
aqui, há uma por linha, porque escolher é o trabalho. Em read-only a lista e a
explicação permanecem e só as ações somem: o que a tela existe para responder
é "o que há para escolher", e isso não depende de poder lançar. Empty state de
verdade é só quando não há nenhuma tarefa disponível. Com dado desatualizado
(princípio 10), cada botão "▶ Começar" some da lista de ações possíveis? Não —
fica visível e desabilitado, com o motivo em texto, porque a lista em si
continua sendo leitura válida do último snapshot; só a ação de lançar não é.

Enquanto uma execução está anexada, a UI entra em **modo de execução** e ocupa
a viewport inteira: as abas Agora/Trabalho desaparecem e não há navegação
lateral. O modo tem dois estados, `em execução` e `concluído`, e é o único
lugar do app onde existe ação de perigo. Sair dele é sempre explícito —
desanexar para segundo plano, ou fechar depois de concluído — nunca por
navegação.

### Controles

- Altura de controles interativos: **32 a 36 px**.
- Raio padrão de cartão: `--radius-3` (12px); controles: `--radius-2` (10px);
  chips e pills: `--radius-1` (8px).
- Borda padrão: `1px solid var(--line)`; destaque: `var(--line-2)`.
- **Selos que dividem uma linha usam a mesma altura, não só a mesma faixa.**
  `--control-height-sm` é um valor único, não um intervalo por componente: o
  `StatusPill`, o selo "Atualizado"/"Desatualizado" e o selo de harness do
  Header ficam lado a lado e por isso têm todos `height: var(--control-height-sm)`
  — nenhum deriva a própria altura de `padding` mais o conteúdo, porque duas
  fontes de altura na mesma linha é o que produz a régua desalinhada. Um selo
  cujo conteúdo pode variar (nome de harness longo, por exemplo) encolhe por
  `text-overflow: ellipsis` dentro dessa altura fixa, nunca crescendo a
  moldura nem esticando por `flex-grow` para preencher a linha — largura segue
  o próprio conteúdo, como os selos vizinhos.

---

## 6. Componentes

### StatusPill
Tom + rótulo textual (nunca só cor). Fundo `-soft`, borda `-line`, tinta do tom.
Ponto de status quando o espaço for mínimo, sempre acompanhado do rótulo.

### HandoffCard
O maior elemento da tela principal, e o único que **carrega o tom do status**
na própria moldura: `in_progress` usa borda `--green-line` sobre um véu
`--green-soft` no `--panel`; `blocked` usa `--amber-line` e `--amber-soft`
(seção 3). O tom na moldura é redundante com o StatusPill do Header de
propósito — nunca substitui o rótulo textual, só torna o objeto central
reconhecível à distância.

Cabeçalho com **proveniência** em destaque — avatar com o ícone do harness
(seção 6, "Lista de harnesses"), e "Escrito no {harness} · {tempo
relativo}" em **sans, tamanho de subtítulo**, não em mono de metadado: é a
primeira frase que a tela responde ("quem parou aqui, e quando"). O timestamp
absoluto, localizado em português por `Intl.DateTimeFormat`, junto dos IDs
(`B-002 / T-002`) vem numa linha mono de metadado logo abaixo. O tempo relativo
usa `Intl.RelativeTimeFormat` no mesmo idioma. Título do objetivo em destaque
(`OBJETIVO · {backlogId} / {todoId}` como rótulo pequeno acima), truncado em
3 linhas (`line-clamp`) — é o resumo de uma frase do que a subtarefa precisa
alcançar, não o lugar para o raciocínio completo; o "Contexto deixado" ao lado
é que existe para isso.

Corpo em **duas colunas lado a lado**, cada uma sua própria caixa (borda
`--line`, fundo `--bg-deep`, `--radius-2`) — "Próximo passo" e "Contexto
deixado" — nunca um parágrafo único misturando os dois. **As duas caixas têm
sempre o mesmo tamanho**: mesma largura (grade `1fr 1fr`) e mesma altura,
porque ambas usam o mesmo teto `--handoff-context-max-height` (seção 5) com
rolagem interna própria. Uma caixa nunca cresce mais que a outra por ter mais
texto; texto que excede rola dentro dela. Assim o card não cresce sem limite
por causa de um contexto longo, e a tela principal continua cabendo numa
viewport sem rolagem de página.

O rótulo de "Próximo passo" leva o **acento do tom** (`--green`, ou `--amber`
quando bloqueado) e o de "Contexto deixado" fica em `--meta`: o acento marca
a ação a tomar, o neutro marca o registro deixado para trás. É a única
distinção de cor entre as duas caixas — a moldura, o fundo e o tamanho são
idênticos.

Rodapé: **um** botão primário ("▶ Retomar {todoId} no {harness}", nomeando o
harness), um botão secundário ("Trocar harness") e o caminho da spec em mono,
alinhado à direita. O card nunca mostra fila, posição ou percentual; mostra o
contador verdadeiro (`2 de 4`) quando houver subtarefas, num rótulo acima da
lista de subtarefas associada, não dentro do card. O botão "▶ Retomar" segue o
princípio 10: desabilita com dado "Desatualizado", porque retomar dispara
execução a partir de um handoff que pode já ter mudado.

### Botão primário
Uma ação primária por tela. Tom sólido (`--green` ou o tom da ação), texto
`--on-green` quando sobre o verde. Variantes: primária (sólida), secundária
(linha `--line-2`), perigo (amber). Estados de hover usam `--raise` ou o
equivalente do tom.

### PreflightModal
Mostra o `argv` **elemento por elemento**, cada um em célula separada (mono),
nunca concatenado com aspas — na prática, uma tabela rotulada de linhas
`bin` / `arg` / `prompt` / `cwd`, cada uma um elemento real do processo a
lançar, nunca uma string montada. O campo `prompt` é editável antes de
confirmar; a edição altera o elemento do `argv`, nunca gera concatenação de
shell. Inclui inline o **seletor de harness e consentimento** (ver
componente abaixo) e o aviso de execução ("Ao executar, esta janela fecha e
o harness assume..."). Rodapé: o escopo de consentimento vigente em texto
("escopo por workspace · gravado local"), um botão secundário ("Cancelar
Esc") e **um** botão primário que nomeia o harness selecionado
("▶ Executar no {harness}"). O botão de confirmar é a única rota de
lançamento. Em `--no-exec`, o modal não existe.

O mesmo modal serve às **quatro portas de lançamento**, mudando só o título e
o `prompt` pré-preenchido: "Especificar uma ideia" (botão "+ nova spec" da
coluna Specs e cartão "Iniciar entrevista" da tela Escolher), "Iniciar sessão
em {backlogId}" (ação "Começar" de uma tarefa disponível) e "Retomar sessão"
(ação "Retomar" do HandoffCard ou de uma tarefa em curso). Não existe rota de
lançamento que não passe por aqui.

As linhas `bin` e `arg` mudam com o harness selecionado, e a linha `prompt`
muda junto: a invocação da skill é **composta para aquele harness** —
`/relay-session` como comando de barra no Claude Code (`claude` `-p`),
`⟨relay-session⟩` como delimitador no Codex (`codex` `exec`) — e nunca uma
flag `--skill`, que não existe em nenhum dos CLIs (`ui-proposal.md`, seção
1.5). O texto da intenção depois do prefixo é o mesmo nos dois; só o prefixo e
o `argv` mudam. Trocar o harness reescreve as quatro linhas e o rótulo do
botão de confirmar na mesma ação.

O rodapé mostra o escopo de consentimento **vigente**, acompanhando a opção
selecionada. Um rodapé fixo dizendo "gravado local" enquanto "Só esta
execução" está marcado é defeito, não estilo: o rodapé é a única confirmação
textual do que será lembrado. O modal fecha por `Esc` e pelo botão "Cancelar";
o seletor standalone, aberto pelo selo do Header, fecha por clique fora.
Fechar nunca lança.

### Seletor de harness e consentimento
Componente compartilhado, usado standalone (atalho "Trocar harness" no
HandoffCard e no Header) e inline no PreflightModal. Lista os harnesses
detectados — nome, versão, estado — e um seletor de **consentimento para
executar sem perguntar de novo**, com três níveis nomeados: "Só esta
execução" (não grava), "Enquanto a app estiver aberta" (grava na sessão),
"Sempre neste workspace" (grava local, por workspace). Consentimento nunca é
selecionado por padrão silenciosamente — a preferência de harness pode ser
local ao workspace, mas não é autorização (ADR-0001 ponto 6): o preflight
continua obrigatório em toda execução, mesmo com consentimento "sempre".

### Painel "Gravado em disco"
A prova de escrita, **ao vivo durante a execução** e não só no fechamento:
ocupa a coluna direita do modo de execução e recebe uma entrada assim que a
skill grava. Expressa a regra read-only como qualidade visível: "o app não
escreveu nada disto; a skill escreveu."

Cabeçalho com o contador verdadeiro de arquivos tocados ("N arquivos
alterados"), nunca percentual. Uma entrada por escrita, **mais recente
primeiro**, cada uma com: um selo de tipo — `ATUALIZADO` quando o registro
passou a ter conteúdo novo, `LIMPO` quando voltou ao estado vazio —, o caminho
do registro em mono, o horário, uma linha de prosa dizendo o que aquela
escrita significa, e as colunas **Antes** e **Depois** lado a lado com o
trecho cru.

A entrada `LIMPO` nunca é omitida por ser "só" um esvaziamento: mostra o
conteúdo anterior à esquerda e o registro vazio à direita, e é exatamente ali
que a invariante do handoff — limpo só depois do changelog — fica visível sem
que ninguém precise explicá-la.

### Cartão de backlog (Trabalho)
Um cartão por entrada de backlog filtrada pela spec selecionada, na coluna
central da aba **Trabalho** — distinto da lista de subtarefas abaixo, que é da
aba **Agora**.

O cartão inteiro assume **um tom**, derivado do marcador: `[•]` → green ("Em
curso"), `[ ]` disponível → blue ("Disponível"), `[ ]` indisponível → meta
("Pendente"), `[!]` → amber ("Bloqueado"), `[x]` → green ("Feito"). O tom
pinta três coisas ao mesmo tempo — a borda do cartão, o `ID` em mono no canto
superior esquerdo e o **rótulo de status** no canto superior direito — e o
fundo recebe o véu `-soft` correspondente. O rótulo de status aqui é **texto
mono em versal, sem chip nem ponto**: o cartão já é a moldura, e um chip
dentro de uma caixa colorida repete a mesma informação três vezes. Continua
valendo a regra da seção 3 (tom **e** texto, nunca só cor) — o que não se usa
aqui é o componente `StatusPill`, que permanece a forma para status fora de
um cartão tonalizado.

Corpo do cartão com o texto da entrada. O rodapé tem sempre o botão secundário
**"Tarefas"**, que abre o modal descrito abaixo — é leitura, não execução, e
por isso existe também em read-only. Quando execução estiver habilitada, o
rodapé ganha ao lado dele o botão que nomeia a ação — `▶ Retomar` (no tom)
para `[•]`, `▶ Começar` (primário sólido) para `[ ]` disponível — desabilitado
com dado "Desatualizado" (princípio 10), igual ao Retomar do HandoffCard e ao
Começar da tela Escolher; "Tarefas" continua sempre clicável, porque é
leitura. O cartão nunca reordena por status: a ordem é a do `BACKLOG.md`
filtrado, invertida (seção 5), e nada além disso a altera — uma tarefa não
sobe por estar em curso nem desce por estar feita.

O cartão é selecionável e a seleção governa a coluna de changelog, então a
área clicável de seleção é o corpo do cartão (ID, status e texto), nunca o
cartão inteiro: um botão dentro de outro botão não existe em HTML, e o rodapé
precisa das próprias ações.

### Modal de tarefas do backlog (Trabalho)
Aberto pelo botão "Tarefas" de um cartão de backlog. Mostra as subtarefas
daquele backlog **na mesma forma da lista de subtarefas da aba Agora**
(marcador + rótulo textual + ID + texto, item atual destacado, mais recente no
topo), porque é a mesma coisa vista de outro lugar — repetir a forma é o que
deixa isso óbvio.

A fonte do dado muda conforme o backlog, e o modal diz qual está usando:

- **Backlog ativo** (o do `TODO.md` corrente): as subtarefas reais, com seus
  marcadores vivos — pendente, em execução, bloqueada, feita — e o contador
  verdadeiro "N de M concluídas".
- **Qualquer outro backlog**: o `TODO.md` já foi substituído, então as
  subtarefas dele só existem como registros de changelog. O modal lista uma
  linha por registro daquele `Backlog:` — todas concluídas por definição, já
  que um registro existir *é* a conclusão (transição 4 do protocolo) — com o
  contador "N executadas". O modal nunca inventa marcador para subtarefa que
  não pode mais ser observada.

Diálogo com `role="dialog"`, `aria-modal`, rótulo, foco preso enquanto aberto
e fechamento por `Esc` e por botão — as mesmas regras do PreflightModal
(seção 7).

O cartão inteiro é clicável (é o próprio controle, não um botão dentro dele) e
seleciona qual backlog filtra a coluna de changelog ao lado — mesma convenção
visual de seleção do cartão de spec (`is-selected`: borda `--blue-line`, fundo
`--blue-soft`). O primeiro cartão da lista filtrada é selecionado por padrão;
trocar de spec ou perder a entrada selecionada reconcilia para o primeiro
disponível, nunca para uma seleção órfã.

### Cartão de changelog (Trabalho)
Um registro por entrada do changelog cujo `Backlog` bate com o cartão de
backlog selecionado na coluna ao lado — nunca o documento inteiro. **Não é uma
caixa**: é uma régua vertical de acento (`--green-line`) à esquerda, sem borda
nem fundo. Changelog é histórico, não trabalho selecionável; dar a ele a mesma
moldura do cartão de backlog sugeriria que se pode agir nele. Cabeçalho em
mono com `{data localizada por Intl} · {T-ID} · {B-ID}`; corpo com o título do registro (a
mesma frase do cabeçalho `## <data> - <T-ID> - <título>` do changelog) em
destaque; rodapé com uma linha mono "evidência: {texto}". Mais recentes primeiro,
na mesma ordem em que `relay-core` devolve os registros (o arquivo já os
mantém assim). Sem cartão de backlog selecionado, a coluna mostra o empty
state convidando a selecionar um; com seleção mas sem registros, mostra que
esse backlog ainda não tem changelog — nunca confunde os dois casos com a
mesma mensagem.

### Lista de subtarefas (checklist)
Cada item combina **marcador + rótulo textual** à direita — `[x]` → "Feito",
`[•]` → "Em execução", `[!]` → "Bloqueado", `[ ]` → "Pendente" — nunca só o
glifo do marcador (mesma regra do StatusPill: tom + texto, nunca só cor). O
item `[•]` ativo tem fundo destacado (`--panel-2` ou tom sutil do status),
distinguindo-o visualmente sem depender só do rótulo. O cabeçalho da lista
mostra o contador verdadeiro ("N de M concluídas"), nunca percentual.

Toda linha tem a **mesma altura**, independente do tamanho do texto da
subtarefa: o texto trunca em duas linhas (`line-clamp`) e o item tem um piso
de altura para as duas linhas cabendo, com o conteúdo centralizado
verticalmente quando ocupa só uma. Uma lista onde cada linha tem a altura do
próprio texto perde o ritmo de leitura — a régua de linhas uniformes é o que
deixa a lista escaneável mesmo com descrições de tamanhos diferentes. A mesma
regra vale para o modal de tarefas do backlog (seção 6, "Modal de tarefas"),
que reusa esta lista.

Na tela principal (aba **Agora**), a lista renderiza em **ordem decrescente**
— a subtarefa mais recente (a atual, `[•]`, quando existir) sempre no topo,
independentemente da ordem textual do `TODO.md`, que continua sem carregar
esse significado (seção 5, "Dependencies" do protocolo). O cabeçalho da lista
fica fixo; o corpo tem altura máxima e rolagem interna própria quando o número
de subtarefas excede o espaço disponível, para que Header, HandoffCard e a
lista caibam juntos numa única viewport sem rolagem de página.

### Terminal
- **Embutido**: `xterm.js` sobre PTY via WebSocket; alt-screen, mouse tracking e
  bracketed paste habilitados; buffer de scrollback por run no host.
- **Externo**: lançado via script wrapper (o script é o contrato, não a linha de
  comando); PID e exit code gravados pelo próprio script.
- Fechar o painel **desanexa**, não mata; reabrir **reanexa** com replay.
- A escolha embutido/externo acontece **antes** do lançamento, dentro do
  próprio preflight: na primeira execução da sessão, confirmar mostra o
  aviso de conflito de teclado (`Cmd+W`, `Cmd+T`) com as duas opções: nenhum
  processo é iniciado até a escolha, e escolher externo nunca chega a
  lançar nem deixar viva uma execução embutida. Runs seguintes na mesma
  sessão lançam embutido direto, sem repetir a pergunta.
- Uma execução já viva encontrada ao reabrir a aba (reanexo automático) não
  passa por essa escolha — não há processo para (re)lançar. Nesse caso, o
  aviso de teclado aparece apenas como reconhecimento (sem opção de modo);
  trocar de modo depois de iniciado é Encerrar processo e lançar de novo.

O terminal nunca aparece sozinho. No modo de execução ele ocupa a coluna
esquerda e o painel "Gravado em disco" ocupa a direita — o Canal A e o Canal B
lado a lado, que é o argumento visual da regra "o estado vem do disco, nunca
do stdout". Acima das duas colunas, uma barra identifica a execução: selo do
harness com o tom de identidade, nome da execução, e o estado da execução em
StatusPill. Ao reanexar, a barra ganha o selo `RECONECTADO À EXECUÇÃO VIVA`,
para que reanexar nunca seja lido como execução nova. O rodapé do terminal
mostra `pid {n} · sessão viva` enquanto o processo vive.

Controles da barra, por estado da execução:

| Estado | Controles |
| --- | --- |
| `em execução` | "Deixar em segundo plano" (secundário) · "Encerrar processo" (perigo) |
| `concluído` | "Fechar" (secundário) |

"Deixar em segundo plano" desanexa e devolve a navegação. **"Encerrar
processo" exige confirmação** — é a única ação de perigo do app e descarta uma
execução que já gravou no disco, com o painel ao lado provando que gravou.
"Fechar" nunca aparece antes de o processo terminar: o rótulo do controle é a
única fonte de verdade sobre o que o clique faz, e o estado na barra nunca
pode dizer `concluído` enquanto a última linha do terminal ainda descreve
trabalho em curso.

### Faixa de execução em segundo plano
Aparece abaixo do Header quando existe execução desanexada, e só então. Ponto
de status, o texto "Rodando em segundo plano", o nome da execução e o harness,
o contador verdadeiro de arquivos já escritos, e um único controle secundário
à direita: "Reconectar ao terminal". A faixa é a prova de que desanexar não é
encerrar — sem ela, "Deixar em segundo plano" seria indistinguível de fechar.

### Lista de harnesses
Um item por harness detectado: nome, versão, estado (instalado / não
autenticado / ausente). "Instalado mas não autenticado" é um estado distinto
de "não instalado".

**Cor por identidade, decidido:** `--purple` para Codex, `--orange` para
Claude Code — nenhum dos dois é usado pelo mapeamento status→tom da seção 3,
então não há colisão de significado com `--green`/`--amber`/`--blue`.
OpenCode "não instalado" não recebe tom de identidade — usa `--meta` sobre
`--line`, mesmo tratamento de qualquer estado desabilitado. Um harness novo
que apareça no futuro entra sem tom próprio (neutro) até o design system
decidir um.

O protótipo v2 **não** implementa esta decisão (ver a divergência na seção 2):
pinta o Codex com `--blue` e o Claude Code com um valor solto, fora de token.
O estado desabilitado do OpenCode aparece lá como `#48525f`, vizinho dos
cinzas proibidos da seção 2 — ali é só o preenchimento de um quadrado, mas
nenhum texto informativo pode herdar esse valor.

**Ícone real em vez de iniciais, decidido em 2026-09-11.** O avatar de 28px
(Header, HandoffCard, seletor de harness, modo de execução) passou a
desenhar o glifo original de cada harness reconhecido — Claude Code, Codex,
OpenCode — em vez das iniciais coloridas. O glifo mantém as cores próprias da
marca (o laranja do Claude Code, o branco+gradiente do Codex); a moldura do
avatar deixa de levar o fundo `-soft`/borda `-line` tonalizados descritos
acima, porque colorir a moldura *e* mostrar o ícone com a cor real da marca
repete a mesma identidade duas vezes — a moldura agora é neutra
(`--panel-2`/`--line`) para qualquer harness com ícone próprio. O OpenCode,
cujo glifo é um traço sólido sem cor de marca, herda `--meta` via
`currentColor`, a mesma regra de "sem tom próprio" que já valia para as
iniciais dele. Um harness sem glifo cadastrado cai num ícone genérico (um
terminal simples), também em `--meta` — nunca inventa iniciais nem tom para
uma marca não reconhecida. `--purple`/`--orange` só existiam no código para
colorir essa moldura (nenhuma outra superfície os usava); com o avatar neutro,
as classes `.harness-avatar--purple`/`--orange` e a função que as escolhia
saem do código. Os tokens continuam declarados na seção 2 como cor de
identidade reservada, para o dia em que outra superfície precisar deles.

### Header
Logotipo "Relay" com nome do workspace e caminho completo, abas **Agora** /
**Trabalho** para alternar tela principal e segunda visão e o estado derivado
em StatusPill à direita. A aba escolhida sobrevive a um reload — guardada em
`sessionStorage`, por aba do browser, nunca na URL: a segunda visão é um
painel, não uma rota, e o non-goal contra roteador client-side continua de pé. Quando execução estiver habilitada, inclui também um
selo compacto do harness ativo — ícone da marca (seção 6, "Lista de
harnesses") e nome — que abre o **seletor de harness e consentimento** ao
clicar. O escopo do consentimento ("não grava" / "sessão" / "workspace")
aparece dentro do seletor, junto de cada opção — repeti-lo no selo compacto do
Header era ruído para quem só quer saber qual harness está ativo. No produto
read-only esse selo não é montado; a proveniência escrita no handoff continua
visível. O Header não carrega ação primária.

### Estado de atualização e conexão

Um rótulo textual compacto, anunciado por uma região de status, distingue os
três estados do dado: **Atualizado**, **Atualizando** e **Desatualizado**. A
transição para Atualizando conserva o último snapshot estável; Desatualizado
significa que a conexão caiu depois de existir dado. O rótulo não bloqueia a
leitura, não usa animação contínua e nunca depende só de cor.

### Empty states
Texto claro quando não há handoff, backlog ou especificação — nunca um painel
vazio sem explicação. Handoff vazio **com** trabalho disponível não é empty
state: é a tela **Escolher** da seção 5, que explica por que a escolha é do
usuário e dá a cada tarefa a própria ação primária. Empty state de verdade é
só quando não há nem trabalho a escolher.

---

## 7. Acessibilidade

- **Contraste AA** em todo texto informativo (ver seção 2).
- **Estado por cor + rótulo** — nunca só por cor.
- **Foco visível** em todos os controles (anel `--blue-line` ou `--line-2`).
- **Alvos de interação** com 32 a 36 px de altura, sem dependência de hover.
- Hierarquia por tamanho e peso, não só por cor.

---

## 8. Implementação

- Tokens definidos como **CSS custom properties** em um arquivo `tokens.css`
  único, importado por todos os módulos de `relay-ui`.
- Componentes consomem apenas tokens; nunca cores ou tamanhos literais fora do
  `tokens.css`.
- Nomenclatura: classe por componente (`status-pill`, `handoff-card`,
  `preflight-modal`, `disk-log`), estados com sufixo (`-is-active`, `-is-muted`).
- A UI é HTML, CSS e JS comuns — sem framework obrigatório. Se um framework
  entrar, os tokens continuam sendo a fonte de estilo.
- O `xterm.js` e seus estilos de terminal ficam isolados no componente Terminal.

---

## 9. Governança

- **Autoridade:** este documento vence sobre protótipos e sobre código.
- **Ordem de mudança:** token ou componente muda primeiro aqui, depois no
  código. Um protótipo não vira token por existir; ele entra aqui por decisão.
- **Estado oficial:** desde 2026-09-11 as seções 2 a 7 descrevem a interface
  implementada, verificada em execução contra um repositório Relay real. Uma
  divergência entre este documento e a tela é defeito de um dos dois e se
  resolve aqui primeiro — não é margem de interpretação. Uma divergência entre
  este documento e um protótipo não é divergência: o protótipo é histórico.
- **Camada:** este documento vive em `docs/design-system/` com a proposta e os
  protótipos. A arquitetura vive na ADR; decisões de desenho e seu raciocínio
  ficam aqui e na proposta. Não duplicar conteúdo entre camadas.
- **Regra invariante da UI:** nada neste documento autoriza a aplicação a
  escrever nos cinco registros do protocolo. Ação de escrita, quando existir,
  é sempre por skill em harness (ADR-0001, ponto 5).

---

## 10. Meio terminal (`relay-tui`)

**Status:** decidida em 2026-10-02 (ADR-0009, spec `20261002-001`), **ainda não
implementada**. As seções 2 a 7 continuam descrevendo só a interface web; esta
seção acrescenta um segundo meio sem substituir nada ali. `tokens.css` não muda.

O `relay-tui` é um painel passivo e estreito, pensado para dividir o terminal
com o harness. Valem para ele os princípios 2, 7, 8, 9 e 10: lê e nunca
escreve, estado nunca só por cor, sem percentual nem fila numerada, sem
superfície de execução, dado antigo se declara antigo. A paleta é própria do
terminal porque um terminal não tem `rgba`, fundo próprio nem fonte escolhida
pela aplicação.

### Fundo e contraste

A TUI **não pinta o fundo**: herda o do terminal, para casar com o painel do
harness ao lado. Por isso o contraste depende de um fundo que o Relay não
controla. Os valores abaixo foram medidos contra `#282c34` (fundo do One Dark)
e a paleta presume terminal escuro; tema claro fica fora da primeira versão.

### Paleta

| Papel | Valor | Contraste | Uso |
| --- | --- | --- | --- |
| `fg` | `#abb2bf` | 6,6:1 | texto primário |
| `meta` | `#9ba6b4` | 5,7:1 | texto secundário e metadados; **é o `--meta` da web**, mantido de propósito |
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

- **`dim` cai na mesma proibição da seção 2.** Com 2,3:1 ele é o mesmo caso dos
  cinzas `#4a5462` e `#5d6675` vetados lá: não atinge AA e não pode carregar
  informação. O One Dark o usa para texto secundário; aqui esse papel é do
  `meta`, que o One Dark não tem em AA (`#828997` dá 4,0:1).
- **Lacuna aceita: `red` com 4,4:1** está abaixo de 4,5:1 sobre `#282c34`, e
  passa em fundos mais escuros (5,2:1 sobre `#1e1e1e`). Mitigação: o vermelho
  nunca está sozinho — vai sempre em negrito, com o glifo e o rótulo
  "Inconsistente"; o corpo das violações usa `fg`.

### Status

| Status | Rótulo | Tom |
| --- | --- | --- |
| `in_progress` | Em andamento | green |
| `blocked` | Bloqueado | yellow |
| `inconsistent` | Inconsistente | **red** |
| `ready` | Pronto | blue |
| `backlog` | A escolher | blue |
| `done` | Concluído | green |
| `idle` | Sem trabalho | `meta` |

Os rótulos são os da seção 3. **Divergência deliberada: `inconsistent` é
vermelho aqui e âmbar na web.** Na web ele divide o âmbar com `blocked`; o
terminal tem um vermelho disponível e o estado que trava o resto da tela ganha
um tom só dele. Tom e rótulo textual continuam obrigatórios (princípio 7).

### Estrutura

- **Cartões** com borda arredondada (`╭ ╮ ╰ ╯`) e o título na própria borda:
  cabeçalho, Handoff, TODO, Spec atual, Specs pendentes e rodapé. A borda é `dim`; a do Handoff herda o
  tom do status. Em `inconsistent`, um cartão de violações (`check` e `detail`)
  substitui o conteúdo.
- **Marcadores do TODO** acompanham o protocolo e sempre trazem texto ao lado:
  `✓` concluído, `●` em andamento, `○` disponível, `◌` indisponível com
  "após T-NNN". Os glifos são Unicode; a primeira versão não define fallback
  ASCII.
- **Barra do TODO** com um segmento por item, no tom do item. É contagem e
  nunca porcentagem (princípio 8; ADR-0003).
- **Frescor em texto:** `atualizado` ou `atualizando`, no cabeçalho.
- **Largura de referência: 58 colunas.** Sem rolagem: faltando altura, o TODO
  trunca em "+N itens" e o backlog (Spec atual e Specs pendentes) reduz-se a uma
  linha de contagem; abaixo de
  cerca de 40 colunas aparecem só o cabeçalho e o status.
- **Teclas:** em Agora, `q` e `Ctrl-C` saem na hora e `Esc` pergunta antes (ver
  "Confirmar a saída"); `Tab` abre a visão Histórico
  e `r` recarrega (ADR-0010; ver "Visão Histórico"). O rodapé mostra só as teclas
  válidas na tela atual.

### View

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

### Visão Histórico

Segunda visão do `relay-tui`, ao lado de **Agora**, que continua sendo a padrão
ao abrir (ADR-0010). O nome diverge de propósito do **Trabalho** da UI web:
Trabalho sugere o que está em curso, que é o que Agora já mostra; Histórico é o
passado e o que falta. É estado local da tela e continua somente leitura: nenhuma
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

### Confirmar a saída

`Esc` em Agora não sai sozinho (um toque perdido encerraria o painel): o rodapé
é trocado pela pergunta `Sair?` (negrito `yellow`) seguida das respostas em `meta`,
`Esc, Enter ou y confirmam · outra tecla cancela`. `Esc`, `Enter` e `y` confirmam;
qualquer outra tecla cancela, o rodapé volta e a tecla é consumida (não faz o que
faria). `q` e `Ctrl-C` são gestos explícitos e saem na hora. Quando a frase não
cabe na largura útil dos cartões (a largura menos quatro), usa-se a mais longa que
cabe: sem `· outra tecla cancela`, depois `Esc ou y confirmam`, `y confirma` e
só `y`; as teclas de resposta nunca são cortadas ao meio. Em Histórico não há
pergunta: `Esc` volta.
