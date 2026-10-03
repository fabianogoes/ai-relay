# 20261002-002 - relay-tui: navegacao pelo historico do trabalho

## Problem

O painel mostra so o agora. Para saber o que ja foi feito (quais specs, quais
itens de backlog, quais tarefas, e o que cada uma entregou) a pessoa abre os
arquivos. Isso deveria caber no proprio painel, em listas curtas que
se aprofundam por selecao: spec, backlog, tarefa, detalhe.

A propria visao Agora tambem nao diz de qual spec e o trabalho: o cartao Backlog
so conta itens do arquivo inteiro (`39 feitos · 4 disponiveis · 6 aguardando`) e,
em `backlog`, lista os disponiveis soltos, sem a spec a que pertencem nem as specs
que ainda tem trabalho pendente.

## Scope

- **Visao Agora agrupada por spec** (esta spec e a dona da mudanca no corpo de
  Agora; a spec 20261002-004 so acrescenta uma linha acima do rodape). O cartao
  Backlog da spec 20261002-001 e substituido por dois cartoes:
  - **Spec atual**: titulo na borda com o id `AAAAMMDD-NNN` e o titulo da spec
    (mesma extracao do nivel de specs, abaixo); a direita, `feitos/total` dos
    itens dela. A spec atual e a do item ativo (o `# Active task` do TODO); sem
    item ativo (estado `backlog`), e a do primeiro item disponivel em ordem
    textual, a recomendacao padrao do protocolo, e o cartao diz `a seguir` em vez
    de `em curso`, sem afirmar prioridade. Dentro: os itens ainda nao feitos da
    spec, na ordem textual, com o mesmo marcador e palavra do cartao Backlog de
    hoje (em curso, disponivel, `aguardando` com `apos B-NNN`, bloqueado); os
    feitos entram so na contagem. Em `backlog` sem nenhuma entrada disponivel nao
    ha spec atual: o cartao nao aparece e todas as specs com pendencia vao para
    Specs pendentes.
  - **Specs pendentes**: uma linha por spec, alem da atual, que tem ao menos um
    item de backlog nao feito, na ordem em que aparecem pela primeira vez no
    `BACKLOG.md`. Linha: id, titulo e `feitos/total`. Itens sem spec valida
    formam a linha **Sem spec**. Sem outra spec pendente, o cartao nao aparece.
  - Em `backlog`, a linha de status `● A escolher` (como a de `ready`) fica no
    lugar do Handoff, e os dois cartoes substituem o cartao `A escolher`. Os
    estados `done`, `idle`, `inconsistent` e "nao e um workspace Relay" nao mudam.
  - **Altura.** A ordem de ceder da spec 20261002-001 continua, com os dois
    cartoes no lugar do Backlog: primeiro Specs pendentes vira uma linha (`N
    specs pendentes`), depois Spec atual corta em `+N itens` e, por fim, os dois
    viram a linha de contagem do Backlog de hoje; so entao o Handoff se compacta
    e o TODO corta.
  - **Largura.** Titulos e itens terminam em `…` em 58 e 40 colunas; abaixo de 40
    colunas vale a linha de status da spec 20261002-001, sem cartoes.

- Uma segunda visao, **Histórico**, ao lado de **Agora**, que continua sendo a
  padrao ao abrir. `Tab` (ou `t`) alterna
  entre as duas, de qualquer nivel; voltar ao Histórico reabre o nivel e a selecao
  em que a pessoa estava.
- Quatro niveis, cada um aprofundando o anterior: **specs**, os **itens de
  backlog** da spec, as **tarefas** do item e o **detalhe** da tarefa. `Enter` ou
  o **clique** numa linha abre o nivel seguinte; as setas, `j`/`k` e a roda do
  mouse movem a selecao; `Esc` ou `Backspace` voltam um nivel e, no nivel de
  specs, voltam para Agora. `Enter` no detalhe nao faz nada; clique fora de uma
  linha (cabecalho, rodape, area vazia) e ignorado.
- Teclas de saida: `q` e `Ctrl-C` saem de qualquer visao. `Esc` so sai em Agora,
  como hoje; em Histórico ele volta (acima).
- Ordem e conteudo de cada nivel:
  - **Specs**: os arquivos de `.specs/`, do mais recente ao mais antigo (ordem
    decrescente do nome de arquivo). Linha: id `AAAAMMDD-NNN`, titulo e a
    contagem `feitos/total` dos seus itens de backlog (nunca porcentagem; `0/0`
    para spec sem itens). O titulo e o texto apos `AAAAMMDD-NNN - ` no primeiro
    `# ` do arquivo; sem esse formato, o texto inteiro do `# `; sem `# `, o nome
    do arquivo. Itens de backlog cuja `spec:` falta ou nao e um arquivo de
    `.specs/` ficam numa ultima linha **Sem spec**.
  - **Itens de backlog**: na ordem textual do `BACKLOG.md`. Linha: id, texto (sem
    as anotacoes `spec:`/`needs:`) e o estado com o mesmo marcador e palavra do
    cartao Backlog de Agora (feito, em curso, disponivel, aguardando, bloqueado).
  - **Tarefas**: os registros do changelog cujo `Backlog` e o item, na ordem
    textual do `CHANGELOG.md`; um `T-NNN` repetido (registro de correcao,
    append-only) aparece de novo como linha propria. Linha: id, titulo e data do
    registro. Se o item e o do TODO atual, os itens do TODO que ainda nao tem
    registro vem depois, com o marcador do TODO (`●`, `○`, `◌`, `!`) e a palavra
    `sem registro`.
  - **Detalhe**: o titulo inteiro e os campos do registro (Backlog, Spec, Result,
    Evidence, Criteria, Decisions), com o texto de continuacao de cada campo
    juntado ao campo; campo ausente (ha registros antigos sem `Criteria`) e
    omitido, nunca inventado. Para uma tarefa sem registro: id, texto e marcador
    do TODO e `Sem registro no changelog ainda.`
- Cada linha de lista ocupa uma linha: o que nao cabe termina em `…`. O detalhe
  quebra o texto pela largura e nunca corta.
- **Rolagem so em Histórico.** A lista rola para manter a selecao visivel e diz,
  em `meta`, quantas linhas ha acima e abaixo; o detalhe rola com setas, `j`/`k`,
  `PgUp`/`PgDn` e a roda. Agora continua sem rolagem (spec 20261002-001).
- **Largura.** Abaixo de 40 colunas, Histórico mostra so o cabecalho, a linha
  `Histórico precisa de 40 colunas` e o rodape; as teclas continuam valendo.
- **Atualizacao.** Quando os registros mudam, as listas sao refeitas e a selecao
  e mantida pelo id (spec, `B-NNN`, posicao do registro); se o item sumiu, a
  selecao vai para a linha mais proxima e, se o nivel inteiro sumiu, sobe ate o
  primeiro nivel que existe. Histórico funciona tambem em estado `inconsistent`
  (a extracao nao depende da derivacao) e, sem `.orchestration/`, mostra a lista
  vazia com `Nenhuma spec em .specs/`.
- **Mouse.** A captura do mouse fica ligada so enquanto o Histórico esta aberto, para
  Agora continuar permitindo selecionar e copiar texto; e desligada ao voltar a
  Agora, ao sair e no panic, junto com a restauracao do terminal.
- **Recarregar.** `r` (em Agora e em Histórico) forca uma releitura do workspace,
  como saida quando o watcher nao entrega eventos (volume de rede, por exemplo);
  a TUI continua se atualizando sozinha e `r` nao e necessario no uso normal. A
  selecao e o nivel sobrevivem pelo id, como em qualquer recarga, e nada e
  escrito. Nao ha teclas F: no macOS elas costumam ser teclas de midia e o
  terminal e a IDE as usam para outras coisas; letras e `Tab` funcionam em
  qualquer terminal e dentro do tmux.
- **Rodape.** Mostra so as teclas validas na tela atual:
  - Agora: `Tab histórico · r recarregar · q sair`.
  - Histórico, nos niveis de lista: `↑↓ mover · Enter abrir · Esc voltar · Tab
    agora · r recarregar · q sair`.
  - Histórico, no detalhe: `↑↓ rolar · Esc voltar · Tab agora · r recarregar · q
    sair` (`Enter` la nao faz nada).
  - **Corte na largura.** Quando o rodape nao cabe, saem indicacoes inteiras (nunca
    meia), da menos para a mais importante: `r recarregar`, `↑↓`, `Enter abrir`,
    `Esc voltar`, `Tab`. `q sair` fica sempre. Em 58 colunas cabem as indicacoes
    de Tab e de `q sair` mais as que sobrarem; em 40, Agora mostra `Tab histórico
    · q sair`.
- Core em Rust: extracao dos campos do changelog (data, titulo e todos os campos,
  inclusive as linhas de continuacao) e do titulo de cada spec. So extracao de
  texto: nenhuma regra nova do protocolo, `RelayState` e os fixtures ficam como
  estao.
- ADR-0004 (detalha a decisao 1 da ADR-0003: as teclas so mudam o que a tela
  mostra), a entrada dela no indice de ADRs do `AGENTS.md` e uma secao no
  `DESIGN.md`.
- Continua somente leitura: a navegacao e estado local da tela; o modulo de
  navegacao nao le disco e entra na lista do teste `read_only.rs`.

## Non-goals

- Escrever, editar ou abrir arquivo no editor; busca e filtro; mostrar o texto
  integral da spec.
- Voltar um nivel pelo mouse; menu ou botoes clicaveis.
- Tornar a extracao do historico regra do protocolo (ver Decisions).
- Tema claro; Windows.

## Decisions

**A tarefa concluida vem do changelog.** O `TODO.md` e esvaziado quando o item de
backlog fecha, entao o unico registro de uma subtarefa feita e o do changelog
(`T-NNN`, titulo, Backlog, Spec, Result, Evidence).

**Emenda a spec 20261002-001.** Aquela spec deixou "navegar, selecionar" e
"changelog e texto de spec na TUI" como nao-objetivos. Isto muda depois do uso
real, como a propria spec previa ("Passivo agora"). O texto de spec continua fora.
A ADR-0004 registra a navegacao.

**Mouse e teclado, os dois.** O pedido e clicar, e a captura do mouse tira a
selecao de texto do terminal; por isso ela so existe na visao Histórico, e todo
gesto do mouse tem equivalente de teclado (acessibilidade e terminais sem mouse).

**Histórico, nao Trabalho.** "Trabalho" sugere o que esta em curso, que e o que
Agora ja mostra; a visao nova e o passado e o que falta. A ADR-0004 registra o
nome.

**`Esc` volta em Histórico e sai em Agora.** Em Histórico, `Esc` e o gesto natural
de voltar; manter nele a saida faria perder a navegacao num toque. Em Agora, o
comportamento da spec 20261002-001 (A-006) fica intacto. No nivel de specs, `Esc`
volta a Agora em vez de sair: sair exige um segundo `Esc` ou `q`.

**Rolagem em Histórico, nao em Agora.** Agora e um painel de olhar, e la a falta
de altura trunca. Uma lista de historico (dezenas de specs e itens neste
repositorio) so e navegavel se rolar; a selecao visivel e a regra.

**Specs do mais recente ao mais antigo.** O historico e consultado a partir do
que acabou de acontecer; dentro de uma spec, a ordem textual do backlog e do
changelog e a do protocolo e nao e reinterpretada.

**A extracao nao e regra do protocolo.** E extracao de texto, coberta por testes
contra os registros reais deste repositorio. Se ela virar regra, o lugar e o
`docs/PROTOCOL.md` primeiro, com fixtures novos.

**Agora agrupada por spec mora aqui.** Ela precisa exatamente do que esta spec
ja cria no core (titulo de cada spec e backlog agrupado por spec, B-041) e do
mesmo vocabulario de linha de spec do nivel de specs; numa spec separada, as duas
extrairiam o mesmo dado. A spec atual sem item ativo e a do primeiro disponivel
porque e a unica escolha que o protocolo ja define como padrao; qualquer outra
(a mais recente, a com mais itens) inventaria uma prioridade.

**Convivencia com a spec 20261002-004.** As duas mudam a visao Agora e os
snapshots dela: esta, o corpo (cartoes de spec) e o rodape; aquela, so uma linha
de proximo passo acima do rodape, cuja regra de altura se refere a "area do
backlog" em qualquer das duas formas. Nao ha dependencia: a que entrar depois
regenera os snapshots de Agora e confere os elementos juntos.

## Acceptance criteria

- A-001 - A ADR-0004 registra a navegacao como estado local da tela, ainda somente
  de leitura, detalhando a decisao 1 da ADR-0003 e emendando os nao-objetivos de
  navegacao da spec 20261002-001; o indice de ADRs do `AGENTS.md` a lista; e o
  `DESIGN.md` descreve a visao Histórico: niveis, ordem, linhas, detalhe,
  rolagem, largura minima, teclas e cliques.
- A-002 - O core em Rust extrai dos registros reais deste repositorio a lista de
  specs com titulo, o backlog por spec (incluindo o grupo Sem spec) e as tarefas
  por item com todos os campos do registro, verificado por testes, que cobrem
  tambem campo com linhas de continuacao, registro sem `Criteria`, `T-NNN`
  repetido e spec sem `# ` no formato esperado; `RelayState` e os fixtures nao
  mudam.
- A-003 - O estado da navegacao, verificado por testes sem terminal: `Tab` e `t`
  alternam Agora e Histórico preservando nivel e selecao; `Enter` ou clique numa
  linha entram no nivel seguinte; `Esc` e `Backspace` voltam um nivel e, no nivel
  de specs, voltam a Agora; `Esc` em Agora, `q` e `Ctrl-C` saem; setas, `j`/`k` e
  roda movem a selecao sem passar dos limites; `r` forca a releitura do workspace
  sem escrever nada e sem perder nivel nem selecao; a selecao sobrevive a uma
  recarga pelo id e se reposiciona quando o item ou o nivel some.
- A-004 - Cada nivel e o detalhe se desenham em 58 e 40 colunas, com linhas
  cortadas com `…`, o detalhe quebrado sem corte, a indicacao de linhas acima e
  abaixo numa lista e num detalhe maiores que a tela, a linha `Histórico precisa
  de 40 colunas` em 30 colunas e o rodape de Agora e de cada nivel de Histórico com
  o texto da decisao, inteiro em 80 colunas e cortado por indicacao inteira em 58
  e 40 colunas, sempre com `q sair` e, em Agora a 40 colunas, `Tab histórico · q
  sair`, verificado por snapshots.
- A-005 - No binario real, num pty: `Tab` liga a captura do mouse e um clique
  (sequencia SGR) numa linha a seleciona e abre o nivel seguinte; voltar a Agora
  e sair desligam a captura; o terminal e restaurado ao sair e o workspace
  continua inalterado.
- A-006 - `app/relay-tui/README.md`, `docs/TUI.md` e `docs/TUI.pt-BR.md`
  descrevem a visao Histórico e suas teclas, inclusive que `Esc` volta em vez de
  sair nela, e a tecla `r` de recarregar.
- A-007 - A visao Agora mostra o cartao Spec atual (id e titulo na borda,
  `feitos/total`, itens nao feitos com marcador e palavra) e, abaixo, o cartao
  Specs pendentes (id, titulo e `feitos/total` por spec, e a linha Sem spec),
  em `in_progress`, `blocked`, `ready` e `backlog` (este com `a seguir` e a spec
  do primeiro item disponivel, e sem o cartao Spec atual quando nao ha entrada
  disponivel), verificado por snapshots em 58 e 40 colunas e por snapshots de
  altura reduzida que mostram a ordem de ceder; os snapshots de `done`, `idle` e
  `inconsistent` continuam sem cartoes de spec.
  A escolha da spec atual usa o `available` vindo do core, sem recalcular
  dependencias. O `DESIGN.md`, `docs/TUI.md` e `docs/TUI.pt-BR.md`
  descrevem os dois cartoes no lugar do cartao Backlog.

## Backlog candidates

- B-001: ADR-0004 e a visao Histórico registradas no `DESIGN.md`.
- B-002: Core em Rust extrai specs, backlog por spec e tarefas por item dos registros.
- B-003: Estado da navegacao (niveis, selecao, teclas e cliques) sem terminal.
- B-004: Listas e detalhe desenhados em 58 e 40 colunas, com snapshots.
- B-005: Binario com captura de mouse na visao Histórico, teste em pty e documentacao.
- B-006: Visao Agora agrupada por spec: spec atual com seus itens e specs pendentes (requer B-002).
